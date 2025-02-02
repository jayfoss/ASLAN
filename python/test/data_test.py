from aslan.aslan_parser import ASLANParser

def test_parses_simple_string_with_no_defaults():
    parser = ASLANParser()
    result = parser.parse('[asland_hi]Hello [asland_lo]World!')
    assert result == {
        '_default': None,
        'hi': 'Hello ',
        'lo': 'World!'
    }

def test_parses_simple_string_with_leading_content():
    parser = ASLANParser()
    result = parser.parse('This is still valid.[asland_hi]Hello [asland_lo]World!')
    assert result == {
        '_default': 'This is still valid.',
        'hi': 'Hello ',
        'lo': 'World!'
    }

def test_parses_simple_string_with_default_append_behavior():
    parser = ASLANParser()
    result = parser.parse('[asland_hi]Hello [asland_lo]World![asland_hi]Hello')
    assert result == {
        '_default': None,
        'hi': 'Hello Hello',
        'lo': 'World!'
    }

def test_parses_simple_string_with_keep_first_key_behavior():
    parser = ASLANParser()
    result = parser.parse('[asland_hi:f]Hello [asland_lo]World![asland_hi]Hello')
    assert result == {
        '_default': None,
        'hi': 'Hello ',
        'lo': 'World!'
    }

def test_parses_simple_string_with_keep_last_key_behavior():
    parser = ASLANParser()
    result = parser.parse('[asland_hi:l]Hello [asland_lo]World![asland_hi]Hello')
    assert result == {
        '_default': None,
        'hi': 'Hello',
        'lo': 'World!'
    }

def test_parses_simple_string_with_keep_first_key_behavior_ignore_duplicate_behavior_redefinition():
    parser = ASLANParser()
    result = parser.parse('[asland_hi:f]Hello [asland_lo]World![asland_hi:a]Hello[asland_hi:l]Test')
    assert result == {
        '_default': None,
        'hi': 'Hello ',
        'lo': 'World!'
    }

def test_parses_simple_string_with_keep_first_key_behavior_on_non_first_key():
    parser = ASLANParser()
    result = parser.parse('[asland_hi]Hello [asland_lo]World![asland_hi:f]Hello[asland_hi:l]Test')
    assert result == {
        '_default': None,
        'hi': 'Hello HelloTest',
        'lo': 'World!'
    }

def test_parses_simple_string_with_keep_last_key_behavior_on_non_first_key():
    parser = ASLANParser()
    result = parser.parse('[asland_hi]Hello [asland_lo]World![asland_hi:l]Hello[asland_hi:f]Test')
    assert result == {
        '_default': None,
        'hi': 'Hello HelloTest',
        'lo': 'World!'
    }

def test_parses_simple_string_with_append_separator():
    parser = ASLANParser(parser_settings={'appendSeparator': ' '})
    result = parser.parse('[asland_hi]Hello[asland_lo]World![asland_hi]Hello[asland_hi]World!')
    assert result == {
        '_default': None,
        'hi': 'Hello Hello World!',
        'lo': 'World!'
    }
