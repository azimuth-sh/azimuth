# Assurance lifecycle experiment

This experiment separates stable evidence qualification from recurring Check Results before either concept enters the Azimuth manifest or assurance-service storage model.

It demonstrates:

- one qualification reused by two CI Check Results over different exact revisions;
- an artifact- and deployment-bound production Check Result;
- injected-time expiry without sleeping;
- deterministic closure for violation, definition drift, Verification Target mismatch and challenge findings;
- temporal confinement that prevents a future qualification from opening an earlier gate;
- zero repository writes and no repeated semantic judgment for ordinary successful executions.

Run `./experiments/assurance-service/check.sh`.
