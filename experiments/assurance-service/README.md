# Assurance lifecycle experiment

This experiment separates stable evidence qualification from recurring execution observations
before either concept enters the Azimuth manifest or assurance-service storage model.

It demonstrates:

- one qualification reused by two CI observations over different exact revisions;
- an artifact- and deployment-bound production observation;
- injected-time expiry without sleeping;
- deterministic closure for violation, definition drift, subject mismatch and challenge findings;
- temporal confinement that prevents a future qualification from opening an earlier gate;
- zero repository writes and no repeated semantic judgment for ordinary successful executions.

Run `./experiments/assurance-service/check.sh`.
