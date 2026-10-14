import unittest

from lita_tokenizers_kytea_py import KyTea


class KyTeaVersionTest(unittest.TestCase):
    def test_native_version(self):
        """Check the C++ -> kytea-sys -> Python pipeline."""
        self.assertEqual(KyTea.version(), "0.4.8")
