from aslan.aslan_parser import ASLANParser

def test_parses_simple_string_with_parts_and_delimiter_like_items():
    parser = ASLANParser()
    result = parser.parse(
        '[asland_test][aslanp]This is an element with [Your name].[aslanp]This is the second part.[aslanp]This is the third part.'
    )
    assert result == {
        '_default': None,
        'test': [
            'This is an element with [Your name].',
            'This is the second part.',
            'This is the third part.',
        ]
    }
