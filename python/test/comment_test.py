from aslan.aslan_parser import ASLANParser

class TestComment:
    def setup_method(self):
        self.parser = ASLANParser()

    def test_parses_simple_string_with_object_comments_ignored(self):
        result = self.parser.parse(
            '[asland_hi]Hello [asland_lo]World![aslanc]This is a comment[asland_foo][aslano][aslanc]This is a comment[asland_bar]Baz!'
        )
        assert result == {
            '_default': None,
            'hi': 'Hello ',
            'lo': 'World!',
            'foo': {
                'bar': 'Baz!'
            }
        }

    def test_parses_more_complex_string_with_object_comments_ignored(self):
        result = self.parser.parse(
            '[asland_hi]Hello [asland_lo]World![aslanc]This is a comment[asland_foo][aslano][aslanc]This is a comment[asland_bar]Baz![aslano][aslanc]This is a comment[asland_x][aslanc]This is a comment[aslano][aslanc]This is a comment[asland_y]you are reading spec[aslanc]This is a comment[asland_z]and it continues here'
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

    def test_parses_more_complex_string_with_object_and_neighbor_aslano_comments_ignored(self):
        result = self.parser.parse(
            '[asland_hi]Hello [asland_lo]World![asland_foo][aslano][aslanc]This is a comment[asland_bar]Baz![aslano][aslanc]This is a comment[asland_x][aslano][aslanc]This is a comment[aslano][aslanc]This is a comment[asland_y]you are reading spec[aslanc]This is a comment[asland_z]and it continues here'
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

    def test_parses_more_complex_string_with_object_and_neighbor_aslano_into_outer_scope_comments_ignored(self):
        result = self.parser.parse(
            '[asland_hi]Hello [asland_lo]World![asland_foo][aslano][aslanc]This is a comment[asland_bar]Baz![aslano][aslanc]This is a comment[asland_x][aslano][aslanc]This is a comment[aslano][aslanc]This is a comment[asland_y]you are reading spec[aslanc]This is a comment[asland_z]and it continues here'
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

    def test_parses_more_complex_string_with_nested_objects_and_arrays_comments_ignored(self):
        result = self.parser.parse(
            '[asland_hi]Hello [asland_lo]World![asland_foo][aslanc]This is a comment[aslano][aslanc]This is a comment[asland_bar]Baz![aslano][aslanc]This is a comment[asland_x][aslano][aslanc]This is a comment[asland_y][aslano][aslanc]This is a comment[asland_z]and it continues here[aslana][aslanc]This is a comment[asland_a][aslanc]This is a comment[aslano]'
        )
        assert result == {
            '_default': None,
            'hi': 'Hello ',
            'lo': 'World!',
            'foo': {
                'bar': 'Baz!'
            },
            'x': {
                'a': {},
                'y': {
                    'z': 'and it continues here'
                }
            }
        }
