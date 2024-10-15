_default:
    @just --list --unsorted --justfile {{justfile()}}

setup:
    pipenv install

dev:
    pipenv run python src/main.py

new-cog:
    python src/dev/new_cog.py

