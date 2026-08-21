# Neutral analyzer fixture

This directory remains in the root experiment sequence as a neutral isolation fixture. It checks
that a well-formed, empty SARIF report can be read without a domain checkout or analyzer install.
The report is ordinary test data; it is not an Azimuth manifest and declares no framework entity.

Run `./experiments/assurance-extensions/check.sh` to validate the fixture's basic SARIF shape.
