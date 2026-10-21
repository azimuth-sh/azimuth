"""Package source links do not grant review outcomes."""

def implements_verification_challenge(identity, *, configuration_files=()):
    def annotate(function):
        function.__azimuth_verification_challenge__ = {'id': identity, 'configuration_files': tuple(configuration_files)}
        return function
    return annotate
