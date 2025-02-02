from aslan.aslan_parser import ASLANParser

class TestCollapseWhitespace:
    def test_parses_array_of_objects_without_whitespace_no_collapse_mode(self):
        parser = ASLANParser(
            {
                'collapseObjectStartWhitespace': False
            }
        )
        result = parser.parse("[asland_array][aslana][asland][aslano][asland_el1]Value 1[asland_el2]Value 2[aslano][asland][aslano][asland_el3]Value 3[asland_el4]Value 4")
        assert result == {
            "_default": None,
            "array": [
                {
                    "el1": "Value 1",
                    "el2": "Value 2"
                },
                {
                    "el3": "Value 3", 
                    "el4": "Value 4"
                }
            ]
        }

    def test_parses_array_of_objects_with_whitespace_collapse_mode(self):
        parser = ASLANParser(
            {
                'collapseObjectStartWhitespace': True
            }
        )
        result = parser.parse("""[asland_array]
      [aslana]
      [asland]
      [aslano]
      [asland_el1]Value 1[asland_el2]Value 2[aslano]
      [asland]
      [aslano]
      [asland_el3]Value 3[asland_el4]Value 4""")
        assert result == {
            "_default": None,
            "array": [
                {
                    "el1": "Value 1",
                    "el2": "Value 2"
                },
                {
                    "el3": "Value 3",
                    "el4": "Value 4"
                }
            ]
        }
