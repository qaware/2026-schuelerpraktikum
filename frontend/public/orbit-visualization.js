/*!
 * OrbitScene — satellite orbit visualisation (SVG + GSAP)
 *
 * Renders Earth at the centre of the scene with N satellites orbiting it.
 * Positions are computed from orbital data, never faked with CSS rotations:
 *
 *   - the orbital period comes from Kepler's third law, T = 2*pi*sqrt(a^3/mu)
 *   - the angular velocity omega = 2*pi/T drives an analytic propagator
 *   - measurements from the JSON correct that propagator instead of replacing it
 *
 * That split is what makes the motion continuous. A satellite is never waiting
 * for the next JSON payload to move: it keeps propagating on its own, and when
 * fresh data arrives the *difference* is eased away with GSAP rather than
 * snapping the satellite to a new spot.
 *
 * Public API:
 *   const scene = OrbitScene.create({ mount: '#orbit-scene', data });
 *   scene.setData(json);      // push new JSON at any time (interpolated, not snapped)
 *   scene.getSatellites();    // processed state, for debugging
 *   scene.destroy();
 *
 * Requires GSAP 3 (window.gsap).
 */
(function (root, factory) {
  'use strict';
  if (typeof module === 'object' && module.exports) module.exports = factory();
  else root.OrbitScene = factory();
})(typeof self !== 'undefined' ? self : this, function () {
  'use strict';

  // ---------------------------------------------------------------------------
  // Physical constants
  // ---------------------------------------------------------------------------

  var EARTH_RADIUS_KM = 6371.0;
  var MU_EARTH = 398600.4418;          // km^3/s^2, Earth's gravitational parameter
  var SIDEREAL_DAY_SEC = 86164.0905;   // Earth's rotation against the stars

  var DEG = Math.PI / 180;
  var TWO_PI = Math.PI * 2;

  // ---------------------------------------------------------------------------
  // Defaults
  // ---------------------------------------------------------------------------

  var DEFAULTS = {
    mount: '#orbit-scene',

    // Data in. Any one of these is enough; they can also be combined.
    data: null,        // object or array, in any shape normalizeSatellites() understands
    dataUrl: null,     // URL polled for fresh JSON
    pollMs: 0,         // >0 enables polling of dataUrl

    // How many simulated seconds pass per real second. Must match the speed-up
    // baked into the generator, otherwise the propagator and the measurements
    // disagree and you get a permanent correction tug-of-war.
    // Set to 'auto' to estimate it from consecutive measurements instead.
    timeScale: 120,
    timeScaleFallback: 120,

    // Viewpoint. Pure camera controls — they change nothing about the orbits
    // themselves, but they decide two things that matter a lot visually:
    //
    //   how much of each orbit passes *behind* Earth (the main depth cue), and
    //   how edge-on each orbit is (too edge-on and it reads as a line, not a
    //   ring — the projected ellipse's minor/major ratio is |orbit normal .
    //   view direction|).
    //
    // These defaults were picked by sweeping both: with the sample set all
    // three orbits go behind Earth while their ellipse ratios stay clearly
    // apart at 0.59 / 0.41 / 0.30. Worth re-checking if your orbits differ a
    // lot in inclination or RAAN.
    cameraElevationDeg: 18,   // above the equatorial plane
    cameraAzimuthDeg: 78,     // around the polar axis

    earthRadius: 92,          // in viewBox units
    orbitBand: [118, 300],    // inner/outer radius the altitude scale maps into

    // How much the radial layout is driven by rank rather than by log distance.
    // 0 = pure log (true relative spacing, but near-identical orbits overlap),
    // 1 = pure rank (evenly spaced rings, no sense of distance at all).
    altitudeMix: 0.55,

    blendSeconds: 1.2,        // how long a measurement correction is eased in

    // Trail length in *real* seconds, sampled at a fixed rate rather than once
    // per frame so its physical length does not change with the frame rate.
    trail: { enabled: true, seconds: 5, sampleHz: 30, maxPoints: 220 },
    stars: { count: 220, twinkle: 28 },
    tooltips: true,
    intro: true,

    palette: ['#38bdf8', '#f472b6', '#4ade80', '#fbbf24', '#a78bfa', '#fb7185'],

    viewBox: [1000, 640]
  };

  // ---------------------------------------------------------------------------
  // Small helpers
  // ---------------------------------------------------------------------------

  function isNum(v) { return typeof v === 'number' && isFinite(v); }
  function clamp(v, lo, hi) { return v < lo ? lo : v > hi ? hi : v; }
  function deepMerge(base, over) {
    var out = {}, k;
    for (k in base) if (Object.prototype.hasOwnProperty.call(base, k)) out[k] = base[k];
    for (k in over) {
      if (!Object.prototype.hasOwnProperty.call(over, k)) continue;
      var v = over[k];
      if (v && typeof v === 'object' && !Array.isArray(v) && out[k] && typeof out[k] === 'object' && !Array.isArray(out[k])) {
        out[k] = deepMerge(out[k], v);
      } else if (v !== undefined) {
        out[k] = v;
      }
    }
    return out;
  }

  /** First argument that is actually a number. Keeps 0 as a valid value. */
  function firstNum() {
    for (var i = 0; i < arguments.length; i++) {
      var v = arguments[i];
      if (isNum(v)) return v;
      if (typeof v === 'string' && v.trim() !== '' && isFinite(Number(v))) return Number(v);
    }
    return undefined;
  }
  function firstStr() {
    for (var i = 0; i < arguments.length; i++) {
      if (typeof arguments[i] === 'string' && arguments[i] !== '') return arguments[i];
    }
    return undefined;
  }

  /** Wrap to [0, 2pi). */
  function wrap(a) { a = a % TWO_PI; return a < 0 ? a + TWO_PI : a; }

  /** Shortest signed difference from `from` to `to`, in (-pi, pi]. */
  function angleDelta(from, to) {
    var d = wrap(to - from);
    return d > Math.PI ? d - TWO_PI : d;
  }

  /** Stable hash of a name, so a satellite without phase data still starts in
   *  the same place on every page load instead of jumping around on reload. */
  function hashAngle(str) {
    var h = 2166136261;
    for (var i = 0; i < str.length; i++) {
      h ^= str.charCodeAt(i);
      h = (h * 16777619) >>> 0;
    }
    return (h / 4294967296) * TWO_PI;
  }

  /** Deterministic PRNG so the starfield is identical on every render. */
  function makeRandom(seed) {
    var s = seed >>> 0;
    return function () {
      s = (s * 1664525 + 1013904223) >>> 0;
      return s / 4294967296;
    };
  }

  // ---------------------------------------------------------------------------
  // Orbital mechanics
  // ---------------------------------------------------------------------------

  /** Semi-major axis of a circular orbit at `altitudeKm`, in km. */
  function semiMajorAxis(altitudeKm) { return EARTH_RADIUS_KM + altitudeKm; }

  /** Kepler's third law. Seconds per revolution. */
  function orbitalPeriod(altitudeKm) {
    var a = semiMajorAxis(altitudeKm);
    return TWO_PI * Math.sqrt((a * a * a) / MU_EARTH);
  }

  /** Circular orbital speed in km/s. */
  function orbitalSpeed(altitudeKm) {
    return Math.sqrt(MU_EARTH / semiMajorAxis(altitudeKm));
  }

  /**
   * Position on a circular orbit, in an Earth-centred inertial frame.
   *
   * `u` is the argument of latitude — the angle travelled along the orbit from
   * the ascending node. This is the same formulation the Python generator uses,
   * so an orbit drawn here matches the sub-satellite points it emits.
   */
  function orbitalVector(radius, inclination, raan, u, out) {
    var cu = Math.cos(u), su = Math.sin(u);
    var ci = Math.cos(inclination), si = Math.sin(inclination);
    var cr = Math.cos(raan), sr = Math.sin(raan);
    out = out || [0, 0, 0];
    out[0] = radius * (cr * cu - sr * su * ci);
    out[1] = radius * (sr * cu + cr * su * ci);
    out[2] = radius * (su * si);
    return out;
  }

  /** Unit vector for a point on the globe. `lon` should already include spin. */
  function surfaceVector(latRad, lonRad, radius, out) {
    var cl = Math.cos(latRad);
    out = out || [0, 0, 0];
    out[0] = radius * cl * Math.cos(lonRad);
    out[1] = radius * cl * Math.sin(lonRad);
    out[2] = radius * Math.sin(latRad);
    return out;
  }

  /**
   * Recover the argument of latitude from a measured latitude.
   *
   * sin(lat) = sin(u) * sin(i) has two solutions per revolution; the ambiguity
   * is resolved by whether the satellite is climbing or descending, which we
   * get from the previous sample. Returns null when the orbit is too close to
   * equatorial for latitude to carry any usable signal.
   */
  function angleFromSample(inclination, sample, previous) {
    if (isNum(sample.phaseDeg)) return wrap(sample.phaseDeg * DEG);

    var sinI = Math.sin(inclination);
    if (!isNum(sample.latitude) || Math.abs(sinI) < 0.05) return null;

    var u = Math.asin(clamp(Math.sin(sample.latitude * DEG) / sinI, -1, 1));
    var ascending = true;
    if (previous && isNum(previous.latitude) && previous.timestamp !== sample.timestamp) {
      ascending = sample.latitude >= previous.latitude;
    }
    return wrap(ascending ? u : Math.PI - u);
  }

  // ---------------------------------------------------------------------------
  // Camera — orthographic projection
  // ---------------------------------------------------------------------------

  /**
   * Camera sitting at elevation `phi` above the equatorial plane, looking at
   * the origin. Screen +x is right, +y is down, and `depth` is positive towards
   * the viewer — that sign is what tells us when something is behind Earth.
   */
  function makeCamera(elevationDeg, azimuthDeg) {
    var phi = elevationDeg * DEG;
    var az = azimuthDeg * DEG;
    return {
      sin: Math.sin(phi), cos: Math.cos(phi),
      sinAz: Math.sin(az), cosAz: Math.cos(az)
    };
  }

  function project(vec, cam, out) {
    // Azimuth is applied as a rotation of the world about the polar axis, which
    // keeps the projection below a single fixed formula.
    var x = vec[0] * cam.cosAz + vec[1] * cam.sinAz;
    var y = -vec[0] * cam.sinAz + vec[1] * cam.cosAz;
    var z = vec[2];

    out = out || { x: 0, y: 0, depth: 0 };
    out.x = x;
    out.y = -(y * cam.sin + z * cam.cos);
    out.depth = -y * cam.cos + z * cam.sin;
    return out;
  }

  /** True when a projected point is hidden by the globe. */
  function isOccluded(p, earthRadius) {
    return p.depth < 0 && (p.x * p.x + p.y * p.y) < earthRadius * earthRadius;
  }

  /**
   * Opacity for something passing behind Earth.
   *
   * A hard cut looks like a glitch, so the last few units before the limb fade
   * out instead. The geometry stays exact; only the transition is softened.
   */
  var FAR_SIDE_OPACITY = 0.55;

  function depthOpacity(p, earthRadius) {
    if (p.depth >= 0) return 1;
    var d = Math.sqrt(p.x * p.x + p.y * p.y);
    if (d >= earthRadius) return FAR_SIDE_OPACITY;   // behind, but clear of the disc
    var t = clamp((earthRadius - d) / 18, 0, 1);     // 0 at the limb, 1 well inside
    return FAR_SIDE_OPACITY - (FAR_SIDE_OPACITY - 0.05) * t;
  }

  // ---------------------------------------------------------------------------
  // SVG path pen — breaks the stroke where it goes behind Earth
  // ---------------------------------------------------------------------------

  function Pen(earthRadius) {
    this.d = '';
    this.down = false;
    this.r = earthRadius;
  }
  Pen.prototype.add = function (p, cx, cy) {
    if (isOccluded(p, this.r)) { this.down = false; return; }
    this.d += (this.down ? 'L' : 'M') + (cx + p.x).toFixed(1) + ' ' + (cy + p.y).toFixed(1) + ' ';
    this.down = true;
  };

  // ---------------------------------------------------------------------------
  // Data normalisation — THE ONLY PLACE THAT KNOWS ABOUT YOUR JSON SHAPE
  //
  // Everything downstream consumes the canonical record produced here:
  //
  //   { name, altitudeKm, inclinationDeg, raanDeg, phaseDeg,
  //     latitude, longitude, timestamp, meta:{...} }
  //
  // To wire up dotage.py, either emit that shape directly or add a branch to
  // toRecords(). No other function needs to change.
  // ---------------------------------------------------------------------------

  /** Pull the satellite array out of whatever wrapper the JSON arrived in. */
  function toRecords(raw) {
    if (!raw) return [];

    if (Array.isArray(raw)) {
      // Either a plain list of satellites, or a list of telemetry log entries.
      if (raw.length && raw[0] && raw[0].position && raw[0].specs) return groupLogEntries(raw);
      return raw;
    }

    if (Array.isArray(raw.satellites)) return raw.satellites;
    if (Array.isArray(raw.sats)) return raw.sats;

    // Telemetry log response: { amount, data: [ { position, specs, timestamp } ] }
    if (Array.isArray(raw.data)) {
      if (raw.data.length && raw.data[0] && raw.data[0].position) return groupLogEntries(raw.data);
      return raw.data;
    }

    // A map of satellite name -> log response / record.
    var keys = Object.keys(raw), out = [], i;
    var looksLikeMap = keys.length > 0 && keys.every(function (k) {
      return raw[k] && typeof raw[k] === 'object';
    });
    if (looksLikeMap) {
      for (i = 0; i < keys.length; i++) {
        var inner = toRecords(raw[keys[i]]);
        for (var j = 0; j < inner.length; j++) {
          if (inner[j].name === undefined) inner[j].name = keys[i];
          out.push(inner[j]);
        }
      }
      return out;
    }
    return [raw];
  }

  /**
   * Collapse telemetry log entries down to one record per satellite.
   *
   * Every sensor on a satellite reports the same position, so the raw list is
   * longer than the number of distinct positions by a factor of the sensor
   * count. Only the newest entry per satellite matters here.
   */
  function groupLogEntries(entries) {
    var newest = Object.create(null);
    for (var i = 0; i < entries.length; i++) {
      var e = entries[i];
      var specs = e.specs || {};
      var name = firstStr(e.name, specs.name, e.satellite);
      if (!name) continue;
      var ts = firstNum(e.timestamp, e.time, 0);
      if (!newest[name] || ts > newest[name]._ts) {
        newest[name] = { _ts: ts, entry: e, name: name };
      }
    }
    return Object.keys(newest).map(function (name) {
      var e = newest[name].entry;
      var pos = e.position || {};
      var specs = e.specs || {};
      return {
        name: name,
        altitudeKm: firstNum(pos.height, pos.altitude),
        inclinationDeg: firstNum(specs.inclination, e.inclination),
        latitude: firstNum(pos.latitude),
        longitude: firstNum(pos.longitude),
        timestamp: newest[name]._ts,
        model: specs.model,
        nation: specs.nation,
        city: pos.city
      };
    });
  }

  /**
   * Canonicalise one record and fill in the derived orbital properties.
   *
   * `index` and `total` only matter for orbits that carry no RAAN: spreading
   * the ascending nodes keeps satellites from being stacked on one line.
   */
  function normalizeRecord(rec, index) {
    var name = firstStr(rec.name, rec.satellite, rec.id, 'SAT-' + (index + 1));
    var altitudeKm = firstNum(
      rec.altitudeKm, rec.altitude_km, rec.altitude, rec.heightKm, rec.height,
      rec.position && rec.position.height, rec.position && rec.position.altitude
    );
    if (!isNum(altitudeKm)) return null;

    var inclinationDeg = firstNum(
      rec.inclinationDeg, rec.inclination, rec.inc,
      rec.specs && rec.specs.inclination
    );
    if (!isNum(inclinationDeg)) inclinationDeg = 0;

    // No RAAN in the data: fan the orbital planes out so they stay legible.
    var raanDeg = firstNum(rec.raanDeg, rec.raan, rec.ascendingNode);
    if (!isNum(raanDeg)) raanDeg = (index * 137.508) % 360;

    return {
      name: name,
      altitudeKm: altitudeKm,
      inclinationDeg: inclinationDeg,
      raanDeg: raanDeg,
      phaseDeg: firstNum(rec.phaseDeg, rec.phase, rec.argumentOfLatitudeDeg, rec.trueAnomalyDeg),
      latitude: firstNum(rec.latitude, rec.lat, rec.position && rec.position.latitude),
      longitude: firstNum(rec.longitude, rec.lon, rec.lng, rec.position && rec.position.longitude),
      timestamp: firstNum(rec.timestamp, rec.time, rec.epoch, 0),
      meta: {
        model: firstStr(rec.model, rec.specs && rec.specs.model),
        nation: firstStr(rec.nation, rec.specs && rec.specs.nation),
        city: firstStr(rec.city, rec.position && rec.position.city)
      }
    };
  }

  /** Public entry point: any supported JSON shape -> canonical records. */
  function normalizeSatellites(raw) {
    var records = toRecords(raw);
    var out = [];
    for (var i = 0; i < records.length; i++) {
      var n = normalizeRecord(records[i] || {}, i);
      if (n) out.push(n);
    }
    return out;
  }

  /**
   * Map altitudes onto the drawable radius band.
   *
   * A LEO satellite at 400 km and a deep-space one at 1.5 million km differ by
   * a factor of ~3700. To scale, either the inner orbits collapse onto Earth or
   * the outer one leaves the frame, so the mapping is logarithmic.
   *
   * Log alone is not enough though: one distant outlier stretches the range so
   * far that everything else piles up at the inner edge. With the sample set,
   * ISS and Hubble — 132 km apart against JWST's 1.5 million — landed on radii
   * 148 and 149, i.e. the same ring. Blending in the rank order guarantees
   * separation while the log term keeps a sense of true relative distance.
   *
   * Either way the radii are not to scale, which is what the legend says.
   */
  function assignOrbitRadii(sats, band, mix) {
    if (!sats.length) return;

    var logs = sats.map(function (s) { return Math.log(semiMajorAxis(s.altitudeKm)); });
    var lo = Math.min.apply(null, logs);
    var hi = Math.max.apply(null, logs);
    var span = hi - lo;

    var order = sats.map(function (_, i) { return i; })
                    .sort(function (a, b) { return logs[a] - logs[b]; });
    var rank = [];
    order.forEach(function (idx, pos) {
      rank[idx] = sats.length > 1 ? pos / (sats.length - 1) : 0.5;
    });

    for (var i = 0; i < sats.length; i++) {
      var logNorm = span > 1e-9 ? (logs[i] - lo) / span : 0.5;
      sats[i].altitudeScale = (1 - mix) * logNorm + mix * rank[i];
      sats[i].orbitRadius = band[0] + (band[1] - band[0]) * sats[i].altitudeScale;
    }
  }

  // ---------------------------------------------------------------------------
  // SVG construction
  // ---------------------------------------------------------------------------

  var SVG_NS = 'http://www.w3.org/2000/svg';

  function svgEl(tag, attrs) {
    var el = document.createElementNS(SVG_NS, tag);
    if (attrs) for (var k in attrs) if (Object.prototype.hasOwnProperty.call(attrs, k)) {
      el.setAttribute(k, attrs[k]);
    }
    return el;
  }

  function buildDefs(uid, earthRadius) {
    var defs = svgEl('defs');
    defs.innerHTML =
      '<radialGradient id="' + uid + '-sky" cx="50%" cy="42%" r="72%">' +
        '<stop offset="0%" stop-color="#0b1220"/><stop offset="100%" stop-color="#01030a"/>' +
      '</radialGradient>' +
      '<radialGradient id="' + uid + '-earth" cx="34%" cy="28%" r="80%">' +
        '<stop offset="0%" stop-color="#5cb2e8"/>' +
        '<stop offset="48%" stop-color="#1b5f9e"/>' +
        '<stop offset="100%" stop-color="#08192a"/>' +
      '</radialGradient>' +
      '<radialGradient id="' + uid + '-night" cx="30%" cy="26%" r="84%">' +
        '<stop offset="40%" stop-color="#000006" stop-opacity="0"/>' +
        '<stop offset="100%" stop-color="#00000c" stop-opacity="0.82"/>' +
      '</radialGradient>' +
      '<radialGradient id="' + uid + '-atmo" cx="50%" cy="50%" r="50%">' +
        '<stop offset="' + (100 * earthRadius / (earthRadius * 1.9)).toFixed(1) + '%" stop-color="#38bdf8" stop-opacity="0"/>' +
        '<stop offset="' + (100 * earthRadius * 1.12 / (earthRadius * 1.9)).toFixed(1) + '%" stop-color="#5ec8ff" stop-opacity="0.34"/>' +
        '<stop offset="100%" stop-color="#38bdf8" stop-opacity="0"/>' +
      '</radialGradient>' +
      '<filter id="' + uid + '-orbitGlow" x="-30%" y="-30%" width="160%" height="160%">' +
        '<feGaussianBlur stdDeviation="3.4" result="b"/>' +
        '<feMerge><feMergeNode in="b"/><feMergeNode in="SourceGraphic"/></feMerge>' +
      '</filter>';
    return defs;
  }

  function buildStarfield(cfg, gsap) {
    var g = svgEl('g', { class: 'os-stars' });
    var rand = makeRandom(0x5eed2026);
    var twinklers = [];
    for (var i = 0; i < cfg.stars.count; i++) {
      var star = svgEl('circle', {
        cx: (rand() * cfg.viewBox[0]).toFixed(1),
        cy: (rand() * cfg.viewBox[1]).toFixed(1),
        r: (0.35 + rand() * 1.15).toFixed(2),
        fill: '#ffffff',
        opacity: (0.18 + rand() * 0.55).toFixed(2)
      });
      g.appendChild(star);
      if (twinklers.length < cfg.stars.twinkle && rand() > 0.55) twinklers.push(star);
    }
    // Twinkle through GSAP rather than CSS keyframes so it shares one timeline
    // and stops cleanly on destroy().
    var tweens = twinklers.map(function (s, i) {
      return gsap.to(s, {
        opacity: 0.12,
        duration: 1.4 + (i % 7) * 0.42,
        repeat: -1,
        yoyo: true,
        ease: 'sine.inOut',
        delay: (i % 11) * 0.31
      });
    });
    return { node: g, tweens: tweens };
  }

  /** Latitude rings never move with the spin, so they are built once. */
  function buildLatitudeRings(cam, earthRadius, cx, cy) {
    var g = svgEl('g', { class: 'os-graticule-lat' });
    var vec = [0, 0, 0], p = { x: 0, y: 0, depth: 0 };
    [-60, -30, 0, 30, 60].forEach(function (lat) {
      var pen = new Pen(earthRadius - 0.4);
      for (var k = 0; k <= 96; k++) {
        surfaceVector(lat * DEG, (k / 96) * TWO_PI, earthRadius, vec);
        project(vec, cam, p);
        pen.add(p, cx, cy);
      }
      g.appendChild(svgEl('path', {
        d: pen.d,
        fill: 'none',
        stroke: '#7dd3fc',
        'stroke-opacity': lat === 0 ? 0.34 : 0.17,
        'stroke-width': lat === 0 ? 0.9 : 0.6
      }));
    });
    return g;
  }

  function buildEarth(uid, cfg, cam) {
    var cx = cfg.viewBox[0] / 2, cy = cfg.viewBox[1] / 2;
    var R = cfg.earthRadius;
    var g = svgEl('g', { class: 'os-earth' });

    g.appendChild(svgEl('circle', { cx: cx, cy: cy, r: R * 1.9, fill: 'url(#' + uid + '-atmo)' }));
    g.appendChild(svgEl('circle', { cx: cx, cy: cy, r: R, fill: 'url(#' + uid + '-earth)' }));
    g.appendChild(buildLatitudeRings(cam, R, cx, cy));

    // Meridians turn with the planet, so they are redrawn every frame.
    var meridians = svgEl('path', {
      class: 'os-graticule-lon', fill: 'none',
      stroke: '#7dd3fc', 'stroke-opacity': '0.17', 'stroke-width': '0.6'
    });
    g.appendChild(meridians);

    g.appendChild(svgEl('circle', { cx: cx, cy: cy, r: R, fill: 'url(#' + uid + '-night)' }));
    g.appendChild(svgEl('circle', {
      cx: cx, cy: cy, r: R, fill: 'none',
      stroke: '#7dd3fc', 'stroke-opacity': '0.5', 'stroke-width': '1'
    }));

    return { node: g, meridians: meridians };
  }

  // ---------------------------------------------------------------------------
  // Satellite view — one per object in the JSON
  // ---------------------------------------------------------------------------

  function buildSatelliteView(sat, color, cfg) {
    var g = svgEl('g', { class: 'os-sat', 'data-name': sat.name });

    var halo = svgEl('circle', { r: 11, fill: color, 'fill-opacity': '0.16' });
    var body = svgEl('circle', { r: 4.6, fill: color });
    var ring = svgEl('circle', {
      r: 4.6, fill: 'none', stroke: '#ffffff',
      'stroke-opacity': '0.85', 'stroke-width': '1.2'
    });
    var label = svgEl('text', {
      y: -15, 'text-anchor': 'middle', class: 'os-sat-label', fill: '#ffffff'
    });
    label.textContent = sat.name;

    // Invisible, generously sized hit area — a 4.6px dot is hard to hover.
    var hit = svgEl('circle', { r: 20, fill: 'transparent', class: 'os-sat-hit' });

    g.appendChild(halo); g.appendChild(body); g.appendChild(ring);
    g.appendChild(label); g.appendChild(hit);

    var trail = null;
    if (cfg.trail.enabled) {
      // Brighter and thicker than the orbit path underneath it (0.4 / 1.3),
      // otherwise the trail is invisible: it runs along exactly the same line.
      trail = svgEl('path', {
        class: 'os-trail', fill: 'none', stroke: color,
        'stroke-opacity': '0.9', 'stroke-width': '2.4',
        'stroke-linecap': 'round', 'stroke-linejoin': 'round'
      });
    }

    return { group: g, trail: trail, label: label, hit: hit };
  }

  /**
   * Split an orbit into the half behind Earth and the half in front of it.
   *
   * Drawing them into two groups — one under the globe, one over it — is what
   * makes an orbit read as a ring around a sphere rather than a flat circle
   * pasted on top. The elements do not change over time, so this runs once.
   */
  function buildOrbitPaths(sat, cam, cfg) {
    var cx = cfg.viewBox[0] / 2, cy = cfg.viewBox[1] / 2;
    var SAMPLES = 256;
    var far = '', near = '';
    var vec = [0, 0, 0], p = { x: 0, y: 0, depth: 0 };
    var farDown = false, nearDown = false;

    for (var k = 0; k <= SAMPLES; k++) {
      orbitalVector(sat.orbitRadius, sat.inclination, sat.raan, (k / SAMPLES) * TWO_PI, vec);
      project(vec, cam, p);
      var x = (cx + p.x).toFixed(1), y = (cy + p.y).toFixed(1);
      if (p.depth >= 0) {
        near += (nearDown ? 'L' : 'M') + x + ' ' + y + ' ';
        nearDown = true; farDown = false;
      } else {
        far += (farDown ? 'L' : 'M') + x + ' ' + y + ' ';
        farDown = true; nearDown = false;
      }
    }
    return { far: far, near: near };
  }

  // ---------------------------------------------------------------------------
  // Tooltip
  // ---------------------------------------------------------------------------

  function buildTooltip(container) {
    var el = document.createElement('div');
    el.className = 'os-tooltip';
    el.setAttribute('role', 'tooltip');
    el.style.opacity = '0';
    el.style.visibility = 'hidden';
    container.appendChild(el);
    return el;
  }

  function tooltipHTML(sat) {
    function row(k, v) {
      return '<div class="os-tt-row"><span class="os-tt-k">' + k + '</span>' +
             '<span class="os-tt-v">' + v + '</span></div>';
    }
    var alt = sat.altitudeKm >= 100000
      ? (sat.altitudeKm / 1e6).toFixed(2) + ' million km'
      : Math.round(sat.altitudeKm).toLocaleString() + ' km';
    var period = sat.periodSec < 10800
      ? (sat.periodSec / 60).toFixed(0) + ' min'
      : sat.periodSec < 259200
        ? (sat.periodSec / 3600).toFixed(1) + ' h'
        : (sat.periodSec / 86400).toFixed(1) + ' days';

    return '<div class="os-tt-title"><span class="os-tt-dot" style="background:' + sat.color + '"></span>' +
             sat.name + '</div>' +
           (sat.meta.model ? row('Model', sat.meta.model) : '') +
           row('Altitude', alt) +
           row('Speed', sat.speedKms.toFixed(2) + ' km/s') +
           row('Period', period) +
           row('Inclination', sat.inclinationDeg.toFixed(2) + '&deg;') +
           (sat.meta.nation ? row('Operator', sat.meta.nation) : '');
  }

  // ---------------------------------------------------------------------------
  // Scene
  // ---------------------------------------------------------------------------

  var uidCounter = 0;

  function create(options) {
    var gsap = (typeof window !== 'undefined') && window.gsap;
    if (!gsap) throw new Error('[OrbitScene] GSAP 3 is required (window.gsap not found).');

    var cfg = deepMerge(DEFAULTS, options || {});
    var container = typeof cfg.mount === 'string' ? document.querySelector(cfg.mount) : cfg.mount;
    if (!container) throw new Error('[OrbitScene] mount element not found: ' + cfg.mount);

    var uid = 'os' + (++uidCounter);
    var cam = makeCamera(cfg.cameraElevationDeg, cfg.cameraAzimuthDeg);
    var cx = cfg.viewBox[0] / 2, cy = cfg.viewBox[1] / 2;

    var sats = [];                 // processed satellite state
    var byName = Object.create(null);
    var destroyed = false;
    var pollTimer = null;
    var hovered = null;
    var introPlayed = false;

    // Auto time-scale estimation state.
    var autoScale = cfg.timeScale === 'auto';
    var timeScale = autoScale ? cfg.timeScaleFallback : cfg.timeScale;

    // --- DOM ----------------------------------------------------------------

    container.classList.add('os-root');
    var svg = svgEl('svg', {
      viewBox: '0 0 ' + cfg.viewBox[0] + ' ' + cfg.viewBox[1],
      class: 'os-svg',
      role: 'img',
      'aria-label': 'Satellites orbiting Earth'
    });
    svg.appendChild(buildDefs(uid, cfg.earthRadius));
    svg.appendChild(svgEl('rect', {
      x: 0, y: 0, width: cfg.viewBox[0], height: cfg.viewBox[1], fill: 'url(#' + uid + '-sky)'
    }));

    var starfield = buildStarfield(cfg, gsap);
    svg.appendChild(starfield.node);

    var orbitsFar = svgEl('g', { class: 'os-orbits os-orbits-far', filter: 'url(#' + uid + '-orbitGlow)' });
    svg.appendChild(orbitsFar);

    var earth = buildEarth(uid, cfg, cam);
    svg.appendChild(earth.node);

    var orbitsNear = svgEl('g', { class: 'os-orbits os-orbits-near', filter: 'url(#' + uid + '-orbitGlow)' });
    svg.appendChild(orbitsNear);

    var trailLayer = svgEl('g', { class: 'os-trails' });
    svg.appendChild(trailLayer);
    var satLayer = svgEl('g', { class: 'os-sats' });
    svg.appendChild(satLayer);

    container.appendChild(svg);
    var tooltip = cfg.tooltips ? buildTooltip(container) : null;

    // Cached geometry for mapping viewBox units to container pixels. Recomputed
    // on resize only — never inside the render loop, which must not read layout.
    var box = { left: 0, top: 0, scale: 1 };
    function measure() {
      var r = svg.getBoundingClientRect();
      var cr = container.getBoundingClientRect();
      box.scale = r.width / cfg.viewBox[0];
      box.left = r.left - cr.left;
      box.top = r.top - cr.top;
    }
    measure();
    var ro = null;
    if (typeof ResizeObserver !== 'undefined') {
      ro = new ResizeObserver(measure);
      ro.observe(container);
    } else if (typeof window !== 'undefined') {
      window.addEventListener('resize', measure);
    }

    // --- Satellite lifecycle ------------------------------------------------

    function makeSatellite(rec, index) {
      var color = cfg.palette[index % cfg.palette.length];
      var view = buildSatelliteView(rec, color, cfg);

      var sat = {
        name: rec.name,
        color: color,
        meta: rec.meta,

        altitudeKm: rec.altitudeKm,
        inclinationDeg: rec.inclinationDeg,
        inclination: rec.inclinationDeg * DEG,
        raan: rec.raanDeg * DEG,

        periodSec: orbitalPeriod(rec.altitudeKm),
        speedKms: orbitalSpeed(rec.altitudeKm),
        angularVelocity: TWO_PI / orbitalPeriod(rec.altitudeKm),  // rad per simulated second

        orbitRadius: 0,       // filled by assignOrbitRadii
        altitudeScale: 0,

        // Propagator state. `currentAngle` = baseAngle + omega*dt + correction.
        baseAngle: isNum(rec.phaseDeg) ? rec.phaseDeg * DEG : hashAngle(rec.name),
        baseSimTime: 0,
        correction: 0,
        currentAngle: 0,
        // Set once the first usable measurement has been adopted. Without it a
        // second payload arriving while simTime is still 0 would be treated as
        // another "first" one and snap the satellite instead of blending.
        initialised: false,

        previousPosition: null,
        nextPosition: null,
        interpolatedPosition: { x: 0, y: 0, z: 0, screenX: 0, screenY: 0, depth: 0, visible: true },

        trailPoints: [],
        view: view,
        orbitFar: svgEl('path', {
          fill: 'none', stroke: color, 'stroke-opacity': '0.14', 'stroke-width': '1.1'
        }),
        orbitNear: svgEl('path', {
          fill: 'none', stroke: color, 'stroke-opacity': '0.4', 'stroke-width': '1.3'
        })
      };

      orbitsFar.appendChild(sat.orbitFar);
      orbitsNear.appendChild(sat.orbitNear);
      if (view.trail) trailLayer.appendChild(view.trail);
      satLayer.appendChild(view.group);

      if (cfg.tooltips) attachTooltip(sat);
      return sat;
    }

    function destroySatellite(sat) {
      gsap.killTweensOf(sat);
      [sat.orbitFar, sat.orbitNear, sat.view.trail, sat.view.group].forEach(function (n) {
        if (n && n.parentNode) n.parentNode.removeChild(n);
      });
    }

    function attachTooltip(sat) {
      sat.view.hit.addEventListener('mouseenter', function () {
        hovered = sat;
        tooltip.innerHTML = tooltipHTML(sat);
        tooltip.style.visibility = 'visible';
        gsap.to(tooltip, { opacity: 1, duration: 0.18, ease: 'power2.out' });
      });
      sat.view.hit.addEventListener('mouseleave', function () {
        if (hovered === sat) hovered = null;
        gsap.to(tooltip, {
          opacity: 0, duration: 0.16, ease: 'power2.in',
          onComplete: function () { if (!hovered) tooltip.style.visibility = 'hidden'; }
        });
      });
    }

    // --- Ingest -------------------------------------------------------------

    /**
     * Feed new JSON in.
     *
     * Existing satellites keep their propagator and receive a *correction*;
     * they are never repositioned directly. New names are added, vanished ones
     * removed, so the scene follows whatever the generator emits.
     */
    function setData(raw) {
      if (destroyed) return;
      var records = normalizeSatellites(raw);
      if (!records.length) return;

      var seen = Object.create(null);
      var structureChanged = false;

      records.forEach(function (rec, i) {
        seen[rec.name] = true;
        var sat = byName[rec.name];
        if (!sat) {
          sat = makeSatellite(rec, sats.length);
          byName[rec.name] = sat;
          sats.push(sat);
          structureChanged = true;
        }
        applyMeasurement(sat, rec);
      });

      // Drop satellites that disappeared from the JSON.
      for (var i = sats.length - 1; i >= 0; i--) {
        if (!seen[sats[i].name]) {
          destroySatellite(sats[i]);
          delete byName[sats[i].name];
          sats.splice(i, 1);
          structureChanged = true;
        }
      }

      if (structureChanged) {
        assignOrbitRadii(sats, cfg.orbitBand, cfg.altitudeMix);
        sats.forEach(function (s) {
          var paths = buildOrbitPaths(s, cam, cfg);
          s.orbitFar.setAttribute('d', paths.far);
          s.orbitNear.setAttribute('d', paths.near);
          s.trailPoints.length = 0;
        });
        // Only on the first build: replaying the draw-in every time a satellite
        // is added would restart the animation on the ones already flying.
        if (cfg.intro && !introPlayed) { introPlayed = true; playIntro(); }
      }
    }

    /**
     * Fold one measurement into a satellite's propagator.
     *
     * The measured angle is propagated forward to *now* and compared with where
     * the satellite is currently drawn. Only that difference is tweened, so the
     * satellite drifts onto the true position over `blendSeconds` instead of
     * jumping to it.
     */
    function applyMeasurement(sat, rec) {
      sat.altitudeKm = rec.altitudeKm;
      sat.periodSec = orbitalPeriod(rec.altitudeKm);
      sat.speedKms = orbitalSpeed(rec.altitudeKm);
      sat.angularVelocity = TWO_PI / sat.periodSec;
      sat.inclinationDeg = rec.inclinationDeg;
      sat.inclination = rec.inclinationDeg * DEG;
      sat.meta = rec.meta;

      var sample = {
        timestamp: rec.timestamp,
        latitude: rec.latitude,
        longitude: rec.longitude,
        phaseDeg: rec.phaseDeg
      };
      var measured = angleFromSample(sat.inclination, sample, sat.nextPosition);
      if (measured === null) { sat.previousPosition = sat.nextPosition; sat.nextPosition = sample; return; }
      sample.angle = measured;

      // Estimate how fast simulated time runs, if asked to.
      if (autoScale && sat.nextPosition && isNum(sat.nextPosition.angle)) {
        var dtReal = sample.timestamp - sat.nextPosition.timestamp;
        if (dtReal > 0.4 && dtReal < 120) {
          var travelled = wrap(sample.angle - sat.nextPosition.angle);
          var implied = travelled / (sat.angularVelocity * dtReal);
          if (isFinite(implied) && implied > 0.5 && implied < 20000) {
            timeScale = timeScale * 0.75 + implied * 0.25;
          }
        }
      }

      sat.previousPosition = sat.nextPosition;
      sat.nextPosition = sample;

      // First measurement: adopt it outright, there is nothing to blend from.
      if (!sat.initialised) {
        sat.baseAngle = measured;
        sat.baseSimTime = simTime;
        sat.correction = 0;
        sat.initialised = true;
        return;
      }

      var propagated = sat.baseAngle + sat.angularVelocity * (simTime - sat.baseSimTime);
      var err = angleDelta(wrap(propagated + sat.correction), measured);

      // power2.inOut, not power2.out: the satellite is already moving, so the
      // correction has to start from zero velocity. An "out" ease applies its
      // fastest motion on the very first frame after the data lands, which
      // reads as a visible tug exactly when a payload arrives.
      gsap.to(sat, {
        correction: sat.correction + err,
        duration: cfg.blendSeconds,
        ease: 'power2.inOut',
        overwrite: 'auto'
      });
    }

    function playIntro() {
      var paths = sats.map(function (s) { return s.orbitNear; })
        .concat(sats.map(function (s) { return s.orbitFar; }));
      paths.forEach(function (p, i) {
        var len = 0;
        try { len = p.getTotalLength(); } catch (e) { len = 0; }
        if (!len) return;
        gsap.fromTo(p,
          { strokeDasharray: len, strokeDashoffset: len },
          {
            strokeDashoffset: 0, duration: 1.1, delay: (i % sats.length) * 0.12,
            ease: 'power2.inOut',
            onComplete: function () { p.style.strokeDasharray = ''; p.style.strokeDashoffset = ''; }
          });
      });
      gsap.from(sats.map(function (s) { return s.view.group; }), {
        opacity: 0, duration: 0.5, delay: 0.35, stagger: 0.1, ease: 'power2.out'
      });
    }

    // --- Render loop --------------------------------------------------------

    var simTime = 0;
    var lastTickerTime = null;
    var vec = [0, 0, 0];
    var proj = { x: 0, y: 0, depth: 0 };
    var meridianPen;

    function updateGlobe() {
      var spin = (simTime / SIDEREAL_DAY_SEC) * TWO_PI;
      meridianPen = new Pen(cfg.earthRadius - 0.4);
      for (var m = 0; m < 12; m++) {
        var lon0 = (m / 12) * TWO_PI + spin;
        meridianPen.down = false;
        for (var k = 0; k <= 40; k++) {
          var lat = -Math.PI / 2 + (k / 40) * Math.PI;
          surfaceVector(lat, lon0, cfg.earthRadius, vec);
          project(vec, cam, proj);
          meridianPen.add(proj, cx, cy);
        }
      }
      earth.meridians.setAttribute('d', meridianPen.d);
    }

    function updateSatellite(sat, now) {
      sat.currentAngle = wrap(
        sat.baseAngle + sat.angularVelocity * (simTime - sat.baseSimTime) + sat.correction
      );

      orbitalVector(sat.orbitRadius, sat.inclination, sat.raan, sat.currentAngle, vec);
      project(vec, cam, proj);

      var ip = sat.interpolatedPosition;
      ip.x = vec[0]; ip.y = vec[1]; ip.z = vec[2];
      ip.screenX = cx + proj.x; ip.screenY = cy + proj.y;
      ip.depth = proj.depth;
      ip.visible = !isOccluded(proj, cfg.earthRadius);

      var g = sat.view.group;
      g.setAttribute('transform', 'translate(' + ip.screenX.toFixed(2) + ',' + ip.screenY.toFixed(2) + ')');
      g.setAttribute('opacity', depthOpacity(proj, cfg.earthRadius).toFixed(3));

      if (sat.view.trail) updateTrail(sat, now);
    }

    var TRAIL_STRIDE = 4;   // x, y, visible, sampledAt

    /**
     * Append to the trail at a fixed rate and drop anything older than
     * `trail.seconds`.
     *
     * Sampling per frame instead would tie the trail's physical length to the
     * frame rate: the same satellite would leave a short trail on a 30 Hz
     * display and a long one at 120 Hz.
     */
    function updateTrail(sat, now) {
      var pts = sat.trailPoints;
      var minGap = 1 / cfg.trail.sampleHz;
      var ip = sat.interpolatedPosition;

      if (!pts.length || now - pts[pts.length - 1] >= minGap) {
        pts.push(ip.screenX, ip.screenY, ip.visible ? 1 : 0, now);
      } else {
        // Between samples, keep the last point glued to the satellite so the
        // trail never visibly lags behind the dot.
        pts[pts.length - 4] = ip.screenX;
        pts[pts.length - 3] = ip.screenY;
        pts[pts.length - 2] = ip.visible ? 1 : 0;
      }

      var cutoff = now - cfg.trail.seconds;
      while (pts.length >= TRAIL_STRIDE && pts[3] < cutoff) pts.splice(0, TRAIL_STRIDE);
      while (pts.length / TRAIL_STRIDE > cfg.trail.maxPoints) pts.splice(0, TRAIL_STRIDE);

      var d = '', down = false;
      for (var i = 0; i < pts.length; i += TRAIL_STRIDE) {
        if (!pts[i + 2]) { down = false; continue; }   // hidden behind Earth
        d += (down ? 'L' : 'M') + pts[i].toFixed(1) + ' ' + pts[i + 1].toFixed(1) + ' ';
        down = true;
      }
      sat.view.trail.setAttribute('d', d);
    }

    var LABEL_BASE_Y = -15;
    var LABEL_STEP = 14;
    var LABEL_MAX_SHIFTS = 6;

    /**
     * Stack labels that would otherwise sit on top of each other.
     *
     * Widths are estimated from the character count rather than measured with
     * getBBox(): a real measurement is a layout read, and doing one per label
     * per frame is exactly the kind of thing that turns a 60 fps scene into a
     * 40 fps one. Satellites nearer the camera keep the preferred slot.
     */
    function declutterLabels() {
      if (sats.length < 2) {
        if (sats.length) sats[0].view.label.setAttribute('y', LABEL_BASE_Y);
        return;
      }

      var ordered = sats.slice().sort(function (a, b) {
        return b.interpolatedPosition.depth - a.interpolatedPosition.depth;
      });
      var placed = [];

      for (var i = 0; i < ordered.length; i++) {
        var s = ordered[i];
        var ip = s.interpolatedPosition;
        var halfW = s.name.length * 3.5 + 5;
        var y = LABEL_BASE_Y;

        for (var shift = 0; shift < LABEL_MAX_SHIFTS; shift++) {
          var clash = false;
          for (var k = 0; k < placed.length; k++) {
            var p = placed[k];
            if (Math.abs(p.x - ip.screenX) < halfW + p.halfW &&
                Math.abs(p.y - (ip.screenY + y)) < LABEL_STEP - 1) {
              clash = true;
              break;
            }
          }
          if (!clash) break;
          y -= LABEL_STEP;
        }

        placed.push({ x: ip.screenX, y: ip.screenY + y, halfW: halfW });
        s.view.label.setAttribute('y', y);
      }
    }

    function updateTooltip() {
      if (!hovered || !tooltip) return;
      var ip = hovered.interpolatedPosition;
      tooltip.style.left = (box.left + ip.screenX * box.scale) + 'px';
      tooltip.style.top = (box.top + ip.screenY * box.scale) + 'px';
    }

    /**
     * One frame. Driven by gsap.ticker so the render, the correction tweens and
     * the twinkle all advance on the same clock — mixing ticker and a private
     * requestAnimationFrame would let them drift apart under load.
     */
    function onTick(time) {
      if (destroyed) return;
      if (lastTickerTime === null) lastTickerTime = time;
      var dt = time - lastTickerTime;
      lastTickerTime = time;
      // gsap.ticker already smooths lag spikes; this only guards a tab that was
      // backgrounded for minutes and would otherwise jump a whole revolution.
      if (dt > 0.5) dt = 0.5;
      simTime += dt * timeScale;

      updateGlobe();
      for (var i = 0; i < sats.length; i++) updateSatellite(sats[i], time);
      declutterLabels();
      updateTooltip();
    }

    gsap.ticker.add(onTick);

    // --- Data sources -------------------------------------------------------

    function fetchOnce() {
      if (!cfg.dataUrl || destroyed) return Promise.resolve();
      return fetch(cfg.dataUrl, { cache: 'no-store' })
        .then(function (r) { return r.json(); })
        .then(function (json) { setData(json); })
        .catch(function (err) { console.warn('[OrbitScene] fetch failed:', err); });
    }

    if (cfg.data) setData(cfg.data);
    if (cfg.dataUrl) {
      fetchOnce();
      if (cfg.pollMs > 0) pollTimer = setInterval(fetchOnce, cfg.pollMs);
    }

    // --- Teardown -----------------------------------------------------------

    function destroy() {
      if (destroyed) return;
      destroyed = true;
      gsap.ticker.remove(onTick);
      starfield.tweens.forEach(function (t) { t.kill(); });
      sats.forEach(destroySatellite);
      sats.length = 0;
      if (pollTimer) clearInterval(pollTimer);
      if (ro) ro.disconnect();
      else if (typeof window !== 'undefined') window.removeEventListener('resize', measure);
      if (tooltip && tooltip.parentNode) tooltip.parentNode.removeChild(tooltip);
      if (svg.parentNode) svg.parentNode.removeChild(svg);
      container.classList.remove('os-root');
    }

    return {
      setData: setData,
      refresh: fetchOnce,
      getSatellites: function () { return sats; },
      getTimeScale: function () { return timeScale; },
      destroy: destroy,
      svg: svg,
      config: cfg
    };
  }

  return {
    create: create,
    normalizeSatellites: normalizeSatellites,
    orbitalPeriod: orbitalPeriod,
    orbitalSpeed: orbitalSpeed,
    EARTH_RADIUS_KM: EARTH_RADIUS_KM,
    MU_EARTH: MU_EARTH
  };
});
