/*
 * GSAP-Animationen fuer das Frontend.
 *
 * Die Animationslogik liegt bewusst hier und nicht in Rust: GSAP-Konfiguration
 * ist verschachtelte JS-Objekte, und die ueber js_sys::Reflect aus Rust
 * zusammenzubauen waere ein Vielfaches an Code fuer dasselbe Ergebnis. Rust
 * ruft nur die benannten Funktionen unten auf.
 *
 * Jede Funktion verschiebt ihre Arbeit selbst in den naechsten Frame. Damit
 * muss die Rust-Seite nicht wissen, ob Leptos das DOM schon eingehaengt hat.
 *
 * Ohne GSAP -- oder wenn der Nutzer reduzierte Bewegung verlangt -- faellt jede
 * Funktion auf den Endzustand zurueck. Die Seite zeigt dann dieselben Daten,
 * nur ohne Bewegung; nichts verschwindet.
 */
(function () {
    "use strict";

    var gsap = window.gsap;
    var reduce = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

    /* Getweent wird nur mit GSAP und ohne Reduced-Motion-Wunsch. Gezeichnet
     * wird immer -- deshalb steht die Pruefung nicht mehr als frueher Ausstieg
     * am Dateianfang, sondern in jeder Funktion an der Stelle, wo es um
     * Bewegung geht. */
    var animate = !!gsap && !reduce;

    if (!gsap) {
        console.warn("[anim] GSAP nicht geladen – Animationen sind deaktiviert.");
    }

    function nextFrame(fn) {
        window.requestAnimationFrame(function () {
            try {
                fn();
            } catch (e) {
                console.error("[anim]", e);
            }
        });
    }

    /* ---------------------------------------------------------------------
     * OrbitScene: die Bahnansicht aus webflow-orbit
     *
     * Die Bahnansicht der Orbitseite. Das Modul in orbit-visualization.js
     * macht die ganze Arbeit selbst: es holt seine Daten per dataUrl, rechnet
     * Umlaufzeit und Geschwindigkeit nach Kepler und laesst einen Propagator
     * laufen, den die Messungen nur korrigieren. Von hier kommt nur Aufbau
     * und Abbau.
     *
     * Deshalb steht hier keine Zeile Bahnmathematik mehr. Die Vorgaenger-
     * ansicht -- Erde von aussen, Bodenspur, in Rust gezeichnetes SVG --
     * brauchte an dieser Stelle eine eigene Interpolation auf der Kugel,
     * weil zwischen zwei Abfragen rund 20 Grad Bahnbogen liegen. Diese Szene
     * bewegt sich pro Frame aus ihrem eigenen Propagator und braucht davon
     * nichts.
     * ------------------------------------------------------------------- */

    var orbitScene = null;

    /* Zaehler statt Flag, weil die Erzeugung einen Frame wartet: verlaesst der
     * Nutzer die Seite in genau diesem Frame, ist orbitScene noch null und
     * orbitSceneDestroy haette nichts zum Aufraeumen. Die hochgezaehlte
     * Generation entwertet die wartende Erzeugung -- sonst bliebe eine Szene
     * uebrig, die unsichtbar weiterpollt. */
    var orbitSceneGen = 0;

    function orbitSceneCreate(mountSelector, optionsJson) {
        orbitSceneDestroy();

        var options;
        try {
            options = JSON.parse(optionsJson);
        } catch (e) {
            console.error("[anim] orbitSceneCreate: ungueltiges JSON", e);
            return;
        }
        options.mount = mountSelector;

        /* Ohne Bewegungswunsch laeuft die Szene mit stehender Simulationszeit:
         * die Satelliten stehen dort, wo die Messung sie hinsetzt, und ruecken
         * bei der naechsten Abfrage ohne Uebergang weiter. Es fehlt also die
         * Bewegung, keine Information. */
        if (!animate) {
            options.timeScale = 0;
            options.blendSeconds = 0;
            options.intro = false;
            options.trail = { enabled: false };
            options.stars = { twinkle: 0 };
        }

        var gen = ++orbitSceneGen;
        nextFrame(function () {
            if (gen !== orbitSceneGen) return;
            if (!window.OrbitScene) {
                console.warn("[anim] OrbitScene nicht geladen – Bahnansicht bleibt leer.");
                return;
            }
            // Leptos haengt den Mount erst ein, nachdem Rust den Effekt
            // ausgeloest hat -- deshalb ueberhaupt der Umweg ueber nextFrame.
            if (!document.querySelector(mountSelector)) return;
            orbitScene = window.OrbitScene.create(options);
        });
    }

    /** Beim Verlassen der Orbitseite: Ticker, Tweens und Polling stoppen. */
    function orbitSceneDestroy() {
        orbitSceneGen++;
        if (!orbitScene) return;
        try {
            orbitScene.destroy();
        } catch (e) {
            console.error("[anim]", e);
        }
        orbitScene = null;
    }

    /* ---------------------------------------------------------------------
     * Allgemeine Seitenanimationen
     * ------------------------------------------------------------------- */

    /* Blendet Elemente ein -- jedes Element aber nur einmal.
     *
     * Das Merken per data-Attribut ist der Grund, warum ein erneutes Rendern
     * der Liste (etwa weil ein Satellit dazukommt) die bereits sichtbaren
     * Karten nicht noch einmal einfliegen laesst. */
    function revealOnce(selector, stagger) {
        nextFrame(function () {
            var els = Array.prototype.slice
                .call(document.querySelectorAll(selector))
                .filter(function (el) {
                    return !el.dataset.animDone;
                });
            if (!els.length) return;

            els.forEach(function (el) {
                el.dataset.animDone = "1";
            });
            if (!animate) return;

            gsap.from(els, {
                opacity: 0,
                y: 26,
                scale: 0.985,
                duration: 0.6,
                ease: "power3.out",
                stagger: stagger,
                // Ohne clearProps bleiben transform/opacity als Inline-Styles
                // stehen und kollidieren spaeter mit den Hover-Effekten.
                clearProps: "opacity,transform",
            });
        });
    }

    /* Zeichnet die Messreihen ein.
     *
     * getTotalLength() liefert die echte Pfadlaenge -- die CSS-Variante musste
     * mit einem festen stroke-dasharray raten und brach bei langen Pfaden ab. */
    function drawPaths(rootSelector, stagger) {
        if (!animate) return;
        nextFrame(function () {
            var root = document.querySelector(rootSelector);
            if (!root) return;

            Array.prototype.forEach.call(
                root.querySelectorAll('path[data-anim="line"]'),
                function (path, i) {
                    var len = path.getTotalLength();
                    if (!len) return;
                    gsap.fromTo(
                        path,
                        { strokeDasharray: len, strokeDashoffset: len },
                        {
                            strokeDashoffset: 0,
                            duration: 1.0,
                            delay: i * stagger,
                            ease: "power2.inOut",
                            onComplete: function () {
                                // Danach wieder eine durchgezogene Linie.
                                path.style.strokeDasharray = "";
                                path.style.strokeDashoffset = "";
                            },
                        }
                    );
                }
            );
        });
    }

    /* Laesst die Messpunkte nacheinander aufpoppen. */
    function popDots(rootSelector, stagger) {
        if (!animate) return;
        nextFrame(function () {
            var root = document.querySelector(rootSelector);
            if (!root) return;
            var dots = root.querySelectorAll('circle[data-anim="dot"]');
            if (!dots.length) return;

            gsap.from(dots, {
                scale: 0,
                opacity: 0,
                duration: 0.3,
                delay: 0.2,
                ease: "back.out(2.5)",
                transformOrigin: "center center",
                stagger: { each: stagger, from: "start" },
                clearProps: "all",
            });
        });
    }

    /* Zaehlt den Messwert-Zaehler auf den neuen Wert hoch.
     *
     * Der Text wird ausschliesslich hier geschrieben, nicht von Leptos --
     * sonst wuerden sich Tween und Reaktivitaet gegenseitig ueberschreiben. */
    function countTo(selector, value) {
        var target = Number(value) || 0;
        nextFrame(function () {
            var el = document.querySelector(selector);
            if (!el) return;

            if (!animate) {
                el.textContent = String(target);
                return;
            }

            var from = Number(String(el.textContent).replace(/[^0-9]/g, "")) || 0;
            if (from === target) return;

            var state = { v: from };
            gsap.killTweensOf(state);
            gsap.to(state, {
                v: target,
                duration: 0.7,
                ease: "power2.out",
                onUpdate: function () {
                    el.textContent = String(Math.round(state.v));
                },
                onComplete: function () {
                    el.textContent = String(target);
                },
            });

            gsap.fromTo(
                el,
                { scale: 1 },
                {
                    scale: 1.16,
                    duration: 0.16,
                    yoyo: true,
                    repeat: 1,
                    ease: "power2.out",
                    transformOrigin: "center center",
                    clearProps: "transform",
                }
            );
        });
    }

    window.satAnim = {
        revealOnce: revealOnce,
        drawPaths: drawPaths,
        popDots: popDots,
        countTo: countTo,
        orbitSceneCreate: orbitSceneCreate,
        orbitSceneDestroy: orbitSceneDestroy,
    };
})();
