import unittest

from azimuth_annotations import covers
from service import identity


class IdentityTests(unittest.TestCase):
    @covers("polyglot/identity", "python-identifies", "unit", "example", "direct")
    def test_identity(self) -> None:
        self.assertEqual(identity(), "python")


if __name__ == "__main__":
    unittest.main()
