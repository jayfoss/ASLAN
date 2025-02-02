from aslan.aslan_parser import ASLANParser

def test_parses_string_with_escape_delimiter():
    parser = ASLANParser()
    result = parser.parse(
        '[asland_example_code][aslane_CODE_BLOCK]function greet(name) {\n'
        '  console.log(`Hello, ${name}!`);\n'
        '  [asland_this_is_not_parsed]This is treated as a regular string[asland_neither_is_this]So is this\n'
        '}[aslane_CODE_BLOCK]'
    )

    assert result == {
        '_default': None,
        'example_code': 'function greet(name) {\n  console.log(`Hello, ${name}!`);\n  [asland_this_is_not_parsed]This is treated as a regular string[asland_neither_is_this]So is this\n}'
    }

def test_parses_string_with_escape_delimiter_ignore_non_matching_close():
    parser = ASLANParser()
    result = parser.parse(
        '[asland_example_code][aslane_CODE_BLOCK]function greet(name) {\n'
        '  console.log(`Hello, ${name}!`);\n'
        '  [asland_this_is_not_parsed]This is treated as a[aslane_other] regular string[asland_neither_is_this]So is this\n'
        '}[aslane_CODE_BLOCK]'
    )

    assert result == {
        '_default': None,
        'example_code': 'function greet(name) {\n  console.log(`Hello, ${name}!`);\n  [asland_this_is_not_parsed]This is treated as a[aslane_other] regular string[asland_neither_is_this]So is this\n}'
    }

def test_parses_string_with_escape_delimiter_continues_parsing_after_escape_closed():
    parser = ASLANParser()
    result = parser.parse(
        '[asland_example][aslane_test][asland_this_is_not_parsed]This is treated as a regular string[aslane_test][asland_this_is_parsed]My value'
    )

    assert result == {
        '_default': None,
        'example': '[asland_this_is_not_parsed]This is treated as a regular string',
        'this_is_parsed': 'My value'
    }
