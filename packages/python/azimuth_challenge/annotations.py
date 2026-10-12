"""Package source links do not grant review outcomes."""

def implements_challenger(identity, *, configuration_files=()):
    def annotate(function):
        function.__azimuth_challenger__ = {'id': identity, 'configuration_files': tuple(configuration_files)}
        return function
    return annotate
