from aslan.aslan_parser import ASLANParser

class TestASLANParserObject:
    def setup_method(self):
        self.parser = ASLANParser()

    def test_parses_simple_string_with_object(self):
        result = self.parser.parse(
            '[asland_hi]Hello [asland_lo]World![asland_foo][aslano][asland_bar]Baz!'
        )
        assert result == {
            '_default': None,
            'hi': 'Hello ',
            'lo': 'World!',
            'foo': {
                'bar': 'Baz!'
            }
        }

    def test_parses_string_with_object_and_comment_between_asland_and_aslano(self):
        result = self.parser.parse(
            '[asland_hi]Hello [asland_lo]World![asland_foo][aslanc]This is a comment[aslano][asland_bar]Baz!'
        )
        assert result == {
            '_default': None,
            'hi': 'Hello ',
            'lo': 'World!',
            'foo': {
                'bar': 'Baz!'
            }
        }

    def test_parses_more_complex_string_with_object(self):
        result = self.parser.parse(
            '[asland_hi]Hello [asland_lo]World![asland_foo][aslano][asland_bar]Baz![aslano][asland_x][aslano][asland_y]you are reading spec[asland_z]and it continues here'
        )
        assert result == {
            '_default': None,
            'hi': 'Hello ',
            'lo': 'World!',
            'foo': {
                'bar': 'Baz!'
            },
            'x': {
                'y': 'you are reading spec',
                'z': 'and it continues here'
            }
        }

    def test_parses_more_complex_string_with_object_and_neighbor_aslano(self):
        result = self.parser.parse(
            '[asland_hi]Hello [asland_lo]World![asland_foo][aslano][asland_bar]Baz![aslano][asland_x][aslano][aslano][asland_y]you are reading spec[asland_z]and it continues here'
        )
        assert result == {
            '_default': None,
            'hi': 'Hello ',
            'lo': 'World!',
            'foo': {
                'bar': 'Baz!'
            },
            'x': {},
            'y': 'you are reading spec',
            'z': 'and it continues here'
        }

    def test_parses_more_complex_string_with_object_and_neighbor_aslano_into_outer_scope(self):
        result = self.parser.parse(
            '[asland_hi]Hello [asland_lo]World![asland_foo][aslano][asland_bar]Baz![aslano][asland_x][aslano][aslano][aslano][asland_y]you are reading spec[asland_z]and it continues here'
        )
        assert result == {
            '_default': None,
            'hi': 'Hello ',
            'lo': 'World!',
            'foo': {
                'bar': 'Baz!'
            },
            'x': {},
            'y': 'you are reading spec',
            'z': 'and it continues here'
        }

    def test_parses_more_complex_string_with_nested_objects(self):
        result = self.parser.parse(
            '[asland_hi]Hello [asland_lo]World![asland_foo][aslano][asland_bar]Baz![aslano][asland_x][aslano][asland_y][aslano][asland_z]and it continues here'
        )
        assert result == {
            '_default': None,
            'hi': 'Hello ',
            'lo': 'World!',
            'foo': {
                'bar': 'Baz!'
            },
            'x': {
                'y': {
                    'z': 'and it continues here'
                }
            }
        }

    def test_parses_simple_string_with_multiple_objects_same_key_should_override(self):
        result = self.parser.parse(
            '[asland_hi][aslano][asland_x]foo[aslano][asland_hi][aslano][asland_y]bar[aslano]'
        )
        assert result == {
            '_default': None,
            'hi': {
                'y': 'bar'
            }
        }

    def test_parses_simple_string_with_object_then_string_same_key_should_not_override_object(self):
        result = self.parser.parse(
            '[asland_hi][aslano][asland_x]foo[aslano][asland_hi]not overriding'
        )
        assert result == {
            '_default': None,
            'hi': {
                'x': 'foo'
            }
        }

    def test_parses_simple_string_with_string_then_object_same_key_should_override_string_with_object(self):
        result = self.parser.parse(
            '[asland_hi]test[asland_hi][aslano][asland_y]bar[aslano]'
        )
        assert result == {
            '_default': None,
            'hi': {
                'y': 'bar'
            }
        }
