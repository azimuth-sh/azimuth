import unittest

from service import identity


class IdentityTests(unittest.TestCase):
    def test_identity(self) -> None:
        self.assertEqual(identity(), "python")


if __name__ == "__main__":
    unittest.main()
