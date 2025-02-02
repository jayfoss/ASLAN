from aslan.aslan_parser import ASLANParser

def test_parses_simple_string_with_object_and_no_go_strict_start():
    parser = ASLANParser({ 'strictStart': True })
    result = parser.parse(
        '[asland_hi]Hello [asland_lo]World![asland_foo][aslano][asland_bar]Baz!'
    )
    assert result == {
        '_default': ''
    }

def test_parses_simple_string_with_object_and_go_strict_start():
    parser = ASLANParser({ 'strictStart': True })
    result = parser.parse(
        '[asland_hi]Hello [aslang][asland_lo]World![asland_foo][aslano][asland_bar]Baz!'
    )
    assert result == {
        '_default': None,
        'lo': 'World!',
        'foo': {
            'bar': 'Baz!'
        }
    }

def test_parses_simple_string_with_object_and_go_strict_start_disabled():
    parser = ASLANParser({ 'strictStart': False })
    result = parser.parse(
        '[asland_hi]Hello [aslang][asland_lo]World![asland_foo][aslano][asland_bar]Baz!'
    )
    assert result == {
        '_default': None,
        'hi': 'Hello ',
        'lo': 'World!',
        'foo': {
            'bar': 'Baz!'
        }
    }

def test_parses_simple_string_with_object_and_go_strict_go_escaped_go():
    parser = ASLANParser({ 'strictStart': True })
    result = parser.parse(
        '[asland_hi]Hello [asland_lo]World![aslane_test][aslang][aslane_test][asland_foo][aslano][asland_bar]Baz!'
    )
    assert result == {
        '_default': '[asland_foo][aslano][asland_bar]Baz!'
    }
