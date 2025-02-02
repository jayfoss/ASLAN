from aslan.aslan_parser import ASLANParser

class TestASLANParserStop:
    def setup_method(self):
        self.parser = ASLANParser(
            {'strictEnd': True}
        )

    def test_parses_simple_string_with_object_and_no_stop_strict_end(self):
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

    def test_parses_simple_string_with_object_and_stop_strict_end(self):
        parser = ASLANParser(
            {'strictEnd': True,
            'multiAslanOutput': True}
        )
        result = parser.parse(
            '[asland_hi]Hello [asland_lo]World![aslans][asland_foo][aslano][asland_bar]Baz!'
        )

        assert result == [
            {
                '_default': None,
                'hi': 'Hello ',
                'lo': 'World!'
            },
            {
                '_default': None,
                'foo': {
                    'bar': 'Baz!'
                }
            }
        ]

    def test_parses_simple_string_with_object_and_stop_strict_end_disabled(self):
        parser = ASLANParser(
            {'strictEnd': False}
        )
        result = parser.parse(
            '[asland_hi]Hello [aslans][asland_lo]World![asland_foo][aslano][asland_bar]Baz!'
        )
        assert result == {
            '_default': None,
            'hi': 'Hello ',
            'lo': 'World!',
            'foo': {
                'bar': 'Baz!'
            }
        }

    def test_parses_simple_string_with_object_and_stop_strict_end_escaped_stop(self):
        result = self.parser.parse(
            '[asland_hi]Hello [asland_lo]World![aslane_test][aslans][aslane_test][asland_foo][aslano][asland_bar]Baz!'
        )
        assert result == {
            '_default': None,
            'hi': 'Hello ',
            'lo': 'World![aslans]',
            'foo': {
                'bar': 'Baz!'
            }
        }
