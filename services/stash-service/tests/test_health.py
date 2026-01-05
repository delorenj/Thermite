"""Health endpoint tests for Persistence Service."""

import pytest
from fastapi.testclient import TestClient

from app.main import app


@pytest.fixture
def client():
    """Test client fixture."""
    return TestClient(app)


def test_health_endpoint(client):
    """Test health check endpoint returns 200 OK."""
    response = client.get("/health")
    assert response.status_code == 200
    data = response.json()
    assert data["status"] == "healthy"
    assert data["service"] == "persistence-service"
    assert "version" in data


def test_security_headers(client):
    """Test security headers are present in responses."""
    response = client.get("/health")
    assert "x-content-type-options" in response.headers
    assert response.headers["x-content-type-options"] == "nosniff"
