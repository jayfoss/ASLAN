from aslan.aslan_parser import ASLANParser

class TestASLANParserQuirks:
    def setup_method(self):
        self.parser = ASLANParser()

    def test_parses_empty_string(self):
        result = self.parser.parse('')
        assert result == {
            '_default': '',
        }

    def test_parses_string_starting_with_delimiter(self):
        result = self.parser.parse('[asland_test]test')
        assert result == {
            '_default': None,
            'test': 'test',
        }
