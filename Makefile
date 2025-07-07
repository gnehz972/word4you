.PHONY: help install run test format lint check dev-install clean

help: ## Show this help message
	@echo "Word4You - Development Commands"
	@echo "================================"
	@echo ""
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

install: ## Install dependencies with uv
	uv sync

run: ## Run the application (usage: make run ARGS="learn beautiful")
	uv run main.py $(ARGS)

test: ## Run tests
	uv run pytest

format: ## Format code with black
	uv run black .

lint: ## Lint code with flake8
	uv run flake8 .

check: ## Run all checks (format, lint, test)
	uv run black --check .
	uv run flake8 .
	uv run pytest

dev-install: ## Install development dependencies
	uv sync --extra dev

clean: ## Clean up generated files
	find . -type f -name "*.pyc" -delete
	find . -type d -name "__pycache__" -delete
	find . -type d -name "*.egg-info" -exec rm -rf {} + 