import unittest
from pathlib import Path

from check_product_icon_resources import ico_entries


class IconContainerTests(unittest.TestCase):
    def setUp(self):
        self.ico = (Path(__file__).resolve().parents[1] / "assets/branding/icons/EffectStrike.ico").read_bytes()

    def test_real_ico_contains_the_nine_declared_png_sizes(self):
        self.assertEqual(len(ico_entries(self.ico)), 9)

    def test_truncated_header_directory_and_payload_are_rejected(self):
        for data in [b"", self.ico[:5], self.ico[:100], self.ico[:-1]]:
            with self.subTest(length=len(data)):
                with self.assertRaises(ValueError):
                    ico_entries(data)

    def test_wrong_directory_dimensions_and_offset_are_rejected(self):
        changed_size = bytearray(self.ico)
        changed_size[6] = 18
        changed_offset = bytearray(self.ico)
        changed_offset[18:22] = (0).to_bytes(4, "little")
        for data in [changed_size, changed_offset]:
            with self.assertRaises(ValueError):
                ico_entries(data)


if __name__ == "__main__":
    unittest.main()
