FROM python:3.13-slim
WORKDIR /app
COPY services/reconciler/reconciler.py /app/reconciler.py
ENV PYTHONDONTWRITEBYTECODE=1 PYTHONUNBUFFERED=1
USER 65532:65532
ENTRYPOINT ["python3", "/app/reconciler.py"]
