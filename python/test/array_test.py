from aslan.aslan_parser import ASLANParser

class TestASLANParserArray:
    def setup_method(self):
        self.parser = ASLANParser()

    def test_parses_simple_string_with_array(self):
        result = self.parser.parse(
            '[asland_fruits][aslana][asland]Apple[asland]Banana[asland]Cherry'
        )
        print(result)
        assert result == {
            '_default': None,
            'fruits': ['Apple', 'Banana', 'Cherry']
        }

    def test_parses_simple_string_with_array_indices(self):
        result = self.parser.parse(
            '[asland_custom_array][aslana][asland_2]Third item[asland_0]First item[asland_1]Second item'
        )
        assert result == {
            '_default': None,
            'custom_array': ['First item', 'Second item', 'Third item']
        }

    def test_parses_simple_string_with_mixed_array_indices(self):
        result = self.parser.parse(
            '[asland_mixed_array][aslana][asland_2]A[asland_0]B[asland]C[asland]D'
        )
        assert result == {
            '_default': None,
            'mixed_array': ['B', None, 'A', 'C', 'D']
        }

    def test_parses_simple_string_with_more_mixed_array_indices(self):
        result = self.parser.parse(
            '[asland_mixed_array][aslana][asland_2]A[asland_0]B[asland]C[asland]D[asland]E[asland_3]F[asland]G'
        )
        assert result == {
            '_default': None,
            'mixed_array': ['B', None, 'A', 'CF', 'D', 'E', 'G']
        }

    def test_parses_simple_string_with_more_mixed_array_indices_and_sibling_object(self):
        result = self.parser.parse(
            '[asland_mixed_array][aslana][asland_2]A[asland_0]B[asland]C[asland]D[asland]E[asland_3]F[asland]G[aslana][asland_other]Not in array'
        )
        assert result == {
            '_default': None,
            'mixed_array': ['B', None, 'A', 'CF', 'D', 'E', 'G'],
            'other': 'Not in array'
        }

    def test_parses_simple_string_with_nested_arrays(self):
        result = self.parser.parse(
            '[asland_mixed_array][aslana][asland_2]A[asland_0]B[asland]C[asland]D[asland]E[asland][aslana][asland]hi[asland]lo[aslana][asland]G'
        )
        assert result == {
            '_default': None,
            'mixed_array': ['B', None, 'A', 'C', 'D', 'E', ['hi', 'lo'], 'G']
        }

    def test_parses_simple_string_with_multiple_arrays_same_key_should_override(self):
        result = self.parser.parse(
            '[asland_hi][aslana][asland]foo[aslana][asland_hi][aslana][asland]bar[aslana]'
        )
        assert result == {
            '_default': None,
            'hi': ['bar']
        }

    def test_parses_simple_string_with_array_then_string_same_key_should_not_override_array(self):
        result = self.parser.parse(
            '[asland_hi][aslana][asland]foo[aslana][asland_hi]not overriding'
        )
        assert result == {
            '_default': None,
            'hi': ['foo']
        }

    def test_parses_simple_string_with_string_then_array_same_key_should_override_string_with_array(self):
        result = self.parser.parse(
            '[asland_hi]test[asland_hi][aslana][asland]bar[aslana]'
        )
        assert result == {
            '_default': None,
            'hi': ['bar']
        }
