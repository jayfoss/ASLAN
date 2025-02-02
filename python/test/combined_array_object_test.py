from aslan.aslan_parser import ASLANParser

class TestCombinedArrayObject:
    def setup_method(self):
        self.parser = ASLANParser()

    def test_parses_simple_string_with_object_in_array(self):
        result = self.parser.parse(
            '[asland_fruits][aslana][asland]Apple[asland]Banana[asland][aslano][asland_x]hi[asland_y]lo'
        )
        assert result == {
            '_default': None,
            'fruits': ['Apple', 'Banana', {'x': 'hi', 'y': 'lo'}]
        }

    def test_parses_simple_string_with_array_in_object(self):
        result = self.parser.parse(
            '[asland_fruits][aslano][asland_best]Apple[asland_worst]Durian[asland_others][aslana][asland]Banana[asland]Pear[aslana][asland_next]Plum'
        )
        assert result == {
            '_default': None,
            'fruits': {
                'best': 'Apple',
                'worst': 'Durian', 
                'others': ['Banana', 'Pear'],
                'next': 'Plum'
            }
        }

    def test_parses_simple_string_with_object_then_array_same_key_should_override_object(self):
        result = self.parser.parse(
            '[asland_hi][aslano][asland_x]foo[aslano][asland_hi][aslana][asland]bar[aslana]'
        )
        assert result == {
            '_default': None,
            'hi': ['bar']
        }

    def test_parses_simple_string_with_array_then_object_same_key_should_override_array(self):
        result = self.parser.parse(
            '[asland_hi][aslana][asland]foo[aslana][asland_hi][aslano][asland_x]bar[aslano]'
        )
        assert result == {
            '_default': None,
            'hi': {'x': 'bar'}
        }
