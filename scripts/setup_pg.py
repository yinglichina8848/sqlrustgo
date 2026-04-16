#!/usr/bin/env python3
import psycopg2
from psycopg2 import sql

conn = psycopg2.connect(
    host="/var/run/postgresql", database="postgres", user="postgres"
)
conn.autocommit = True
cur = conn.cursor()

# Create openclaw role if not exists
cur.execute("SELECT 1 FROM pg_roles WHERE rolname = 'openclaw'")
if not cur.fetchone():
    cur.execute("CREATE ROLE openclaw WITH LOGIN PASSWORD 'openclaw123'")
    print("Created openclaw role")
else:
    print("openclaw role already exists")

# Create database if not exists
cur.execute("SELECT 1 FROM pg_database WHERE datname = 'tpch_test'")
if not cur.fetchone():
    cur.execute("CREATE DATABASE tpch_test")
    print("Created tpch_test database")
else:
    print("tpch_test database already exists")

# Grant privileges
cur.execute("GRANT ALL PRIVILEGES ON DATABASE tpch_test TO openclaw")

print("PostgreSQL setup complete")
conn.close()
