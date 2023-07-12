#!/usr/bin/env python3
import sys
import click


@click.group()
def mate():
    """aikido"""


@mate.command()
@click.argument("parts", multiple=True)
def slugify(parts):
    if not sys.stdin.isatty():
        parts = [line.split() for line in sys.stdin.readlines()]

    print(parts)


if __name__ == "__main__":
    mate()
