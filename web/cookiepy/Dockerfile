FROM pypy:3
WORKDIR /app

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY main.py .

CMD ["gunicorn", "-k", "gevent", "--workers", "4", "--preload", "--bind", "0.0.0.0:12345", "main:app"]
