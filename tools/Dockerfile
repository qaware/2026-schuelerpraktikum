FROM python:3.13-slim

WORKDIR /app

COPY requirements.txt ./
RUN pip install --no-cache-dir -r requirements.txt

COPY DataGenerator.py ./

# Im Compose-Netz heisst das Backend "backend", nicht "localhost".
ENV INGEST_URL=http://backend:8585/data

# -u: ohne Buffering, sonst bleiben die "Sent ..."-Zeilen in 'docker compose
# logs' minutenlang haengen.
CMD ["python", "-u", "DataGenerator.py"]
