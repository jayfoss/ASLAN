from aslan.aslan_parser import ASLANParser

class TestASLANParserVoid:
    def setup_method(self):
        self.parser = ASLANParser()

    def test_parses_simple_string_with_object_and_void_after_other_content(self):
        result = self.parser.parse(
            '[asland_hi]Hello [asland_lo]World![asland_foo][aslano][asland_bar]Baz![aslanv]'
        )
        assert result == {
            '_default': None,
            'hi': 'Hello ',
            'lo': 'World!',
            'foo': {
                'bar': None
            }
        }

    def test_parses_simple_string_with_object_and_void_before_other_content(self):
        result = self.parser.parse(
            '[asland_hi]Hello [asland_lo]World![asland_foo][aslano][asland_bar][aslanv]Baz!'
        )
        assert result == {
            '_default': None,
            'hi': 'Hello ',
            'lo': 'World!',
            'foo': {
                'bar': None
            }
        }

    def test_parses_simple_string_with_object_and_void_in_a_later_duplicate_key_field(self):
        result = self.parser.parse(
            '[asland_hi]Hello [asland_lo]World![asland_foo][aslano][asland_bar]Baz![asland_bar][aslanv]'
        )
        assert result == {
            '_default': None,
            'hi': 'Hello ',
            'lo': 'World!',
            'foo': {
                'bar': None
            }
        }

    def test_parses_simple_string_with_object_and_void_in_a_previous_duplicate_key_field(self):
        result = self.parser.parse(
            '[asland_hi]Hello [asland_lo]World![asland_foo][aslano][asland_bar][aslanv][asland_x]hi[asland_bar]oops'
        )
        assert result == {
            '_default': None,
            'hi': 'Hello ',
            'lo': 'World!',
            'foo': {
                'bar': None,
                'x': 'hi'
            }
        }
