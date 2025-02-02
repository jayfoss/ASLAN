from aslan.aslan_parser import ASLANParser, ASLANInstruction, ASLANEndDataInstruction
from aslan.utils import deep_copy

def test_parses_simple_string_with_instructions():
    parser = ASLANParser()
    content_events = []
    end_events = []
    end_data_events = []

    def on_content(event):
        content_events.append(deep_copy(event))
    
    def on_end(event):
        end_events.append(deep_copy(event))

    def on_end_data(event):
        end_data_events.append(deep_copy(event))

    parser.add_event_listener('content', on_content)
    parser.add_event_listener('end', on_end)
    parser.add_event_listener('end_data', on_end_data)

    result = parser.parse(
        '[asland_styled_text][aslanp][aslani_bold][aslani_color:red]This is bold and red text.[aslanp][aslani_italic][aslani_underline]This is italic and underlined text.[aslanp][aslani_size:large][aslani_font:monospace]This is large monospace text.'
    )

    assert content_events == snapshot('instruction-events-content')
    assert end_events == snapshot('instruction-events-end')
    assert end_data_events == snapshot('instruction-events-end-data')
    assert result == {
        '_default': None,
        'styled_text': [
            'This is bold and red text.',
            'This is italic and underlined text.',
            'This is large monospace text.',
        ]
    }

def test_parses_simple_string_with_instructions_and_data_with_l_arg():
    parser = ASLANParser()
    content_events = []
    end_events = []
    end_data_events = []

    def on_content(event):
        content_events.append(deep_copy(event))
    
    def on_end(event):
        end_events.append(deep_copy(event))

    def on_end_data(event):
        end_data_events.append(deep_copy(event))

    parser.add_event_listener('content', on_content)
    parser.add_event_listener('end', on_end)
    parser.add_event_listener('end_data', on_end_data)

    result = parser.parse(
        '[asland_styled_text:l][aslanp][aslani_bold][aslani_color:red]This is bold and red text.[asland_styled_text][aslanp][aslani_italic][aslani_color:blue]This is italic and blue text.'
    )

    assert content_events == snapshot('instruction-events-content')
    assert end_events == snapshot('instruction-events-end')
    assert end_data_events == snapshot('instruction-events-end-data')
    assert result == {
        '_default': None,
        'styled_text': ['This is italic and blue text.']
    }

def test_parses_simple_string_with_instructions_and_data_with_f_arg():
    parser = ASLANParser()
    content_events = []
    end_events = []
    end_data_events = []

    def on_content(event):
        content_events.append(deep_copy(event))
    
    def on_end(event):
        end_events.append(deep_copy(event))

    def on_end_data(event):
        end_data_events.append(deep_copy(event))

    parser.add_event_listener('content', on_content)
    parser.add_event_listener('end', on_end)
    parser.add_event_listener('end_data', on_end_data)

    result = parser.parse(
        '[asland_styled_text:f][aslanp][aslani_bold][aslani_color:red]This is bold and red text.[asland_styled_text][aslanp][aslani_italic][aslani_color:blue]This is italic and blue text.'
    )

    assert content_events == snapshot('instruction-events-content')
    assert end_events == snapshot('instruction-events-end')
    assert end_data_events == snapshot('instruction-events-end-data')
    assert result == {
        '_default': None,
        'styled_text': ['This is bold and red text.']
    }

def test_parses_simple_string_with_instructions_no_part():
    parser = ASLANParser()
    content_events = []
    end_events = []
    end_data_events = []

    def on_content(event):
        content_events.append(deep_copy(event))
    
    def on_end(event):
        end_events.append(deep_copy(event))

    def on_end_data(event):
        end_data_events.append(deep_copy(event))

    parser.add_event_listener('content', on_content)
    parser.add_event_listener('end', on_end)
    parser.add_event_listener('end_data', on_end_data)

    result = parser.parse(
        '[asland_test][aslana][asland][aslano][asland_styled_text][aslani_bold][aslani_color:red]This is bold and red text.[asland_next]Next item'
    )

    assert content_events == snapshot('instruction-events-content')
    assert end_events == snapshot('instruction-events-end')
    assert end_data_events == snapshot('instruction-events-end-data')
    assert result == {
        '_default': None,
        'test': [
            {
                'next': 'Next item',
                'styled_text': 'This is bold and red text.'
            }
        ]
    }
