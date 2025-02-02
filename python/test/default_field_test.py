from aslan.aslan_parser import ASLANParser

def test_parses_simple_string_with_renamed_default_field():
    parser = ASLANParser(parser_settings={'defaultFieldName': '_modified'})
    result = parser.parse('[asland_hi]Hello [asland_lo]World!')
    assert result == {
        '_modified': None,
        'hi': 'Hello ',
        'lo': 'World!'
    }
