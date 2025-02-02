from aslan.aslan_parser import ASLANParser

def setup_parser():
    return ASLANParser(
        {
            'multiAslanOutput': True,
            'strictStart': True,
            'strictEnd': True
        }
    )

def test_parses_simple_multi_aslan_string():
    parser = setup_parser()
    result = parser.parse(
        '[aslang][asland_hi]Hello [asland_lo]World![aslans][aslang][asland_foo][aslano][asland_bar]Baz![aslans][aslang]Starting again[aslang]And again[aslang][asland_further]This should also work[aslang]And this'
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
        },
        {
            '_default': 'Starting again'
        },
        {
            '_default': 'And again'
        },
        {
            '_default': None,
            'further': 'This should also work'
        },
        {
            '_default': 'And this'
        }
    ]

def test_parses_multi_aslan_string_with_no_content():
    parser = setup_parser()
    result = parser.parse('[aslang][aslang]')
    assert result == [
        {
            '_default': ''
        },
        {
            '_default': ''
        }
    ]

def test_parses_multi_aslan_string_with_content_between_stop_and_go_delimiters():
    parser = setup_parser()
    result = parser.parse(
        '[aslang]This is a test[aslans]this should be ignored[aslang]but not this'
    )
    assert result == [
        {
            '_default': 'This is a test'
        },
        {
            '_default': 'but not this'
        }
    ]
