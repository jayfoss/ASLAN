from aslan.aslan_parser import ASLANParser

class TestASLANParserPart:
    def setup_method(self):
        self.parser = ASLANParser()

    def test_parses_simple_string_with_parts(self):
        result = self.parser.parse(
            '[asland_formatted_text][aslanp]This is the first part.[aslanp]This is the second part.[aslanp]This is the third part.'
        )
        assert result == {
            '_default': None,
            'formatted_text': [
                'This is the first part.',
                'This is the second part.',
                'This is the third part.',
            ]
        }

    def test_parses_simple_string_with_parts_and_instructions(self):
        result = self.parser.parse(
            '[asland_styled_text][aslanp][aslani_bold][aslani_color:red]This is bold and red text.[aslanp][aslani_italic][aslani_underline]This is italic and underlined text.[aslanp][aslani_size:large][aslani_font:monospace]This is large monospace text.'
        )
        assert result == {
            '_default': None,
            'styled_text': [
                'This is bold and red text.',
                'This is italic and underlined text.',
                'This is large monospace text.',
            ]
        }

    def test_parses_simple_string_with_parts_and_text_before_first_part(self):
        result = self.parser.parse(
            '[asland_formatted_text]This is some preamble.[aslanp]This is the first part.[aslanp]This is the second part.[aslanp]This is the third part.'
        )
        assert result == {
            '_default': None,
            'formatted_text': [
                'This is some preamble.',
                'This is the first part.',
                'This is the second part.',
                'This is the third part.',
            ]
        }
