from enum import Enum
from typing import Dict, List, Union, Optional, Callable, TypedDict, Literal
from aslan.recent_items import RecentItems
import re
from aslan.utils import generate_random_idempotency_key
ASLANValue = Union[str, Dict[str, 'ASLANValue'], List['ASLANValue'], None]
ASLANObject = Dict[str, ASLANValue]
ASLANArray = List[ASLANValue]

class InstructionContent(TypedDict):
    value: str
    partIndex: int
    instructions: List[Dict[str, Union[str, List[str], int]]]

class ASLANInstruction(TypedDict):
    content: str
    partIndex: int
    fieldName: Union[str, int]
    path: List[str]
    structure: Union[ASLANObject, ASLANArray]
    instruction: str
    args: List[str]
    index: int
    multiAslanIndex: int
    tag: Literal['content', 'end']

class ASLANEndDataInstruction(TypedDict):
    content: List[InstructionContent]
    fieldName: Union[str, int]
    path: List[str]
    structure: Union[ASLANObject, ASLANArray]
    multiAslanIndex: int
    tag: Literal['end_data']

class ASLANDelimiterType(Enum):
    DATA = 'DATA'
    OBJECT = 'OBJECT'
    INSTRUCTION = 'INSTRUCTION'
    ARRAY = 'ARRAY'
    COMMENT = 'COMMENT'
    ESCAPE = 'ESCAPE'
    PART = 'PART'
    VOID = 'VOID'
    GO = 'GO'
    STOP = 'STOP'

ASLANDuplicateKeyBehavior = Literal['a', 'f', 'l']

ASLANEventHandler = Callable[[Union[ASLANInstruction, ASLANEndDataInstruction]], None]

class ASLANDelimiterData(TypedDict):
    prefix: Optional[str]
    suffix: Optional[ASLANDelimiterType]
    content: Optional[str]
    args: List[str]

class ASLANParserState(Enum):
    GO_DELIMITER = 'GO_DELIMITER'
    STOP_DELIMITER = 'STOP_DELIMITER'
    START = 'START'
    MAYBE_DELIMITER = 'MAYBE_DELIMITER'
    DELIMITER = 'DELIMITER'
    RESERVED_DELIMITER = 'RESERVED_DELIMITER'
    OBJECT = 'OBJECT'
    ARRAY = 'ARRAY'
    COMMENT = 'COMMENT'
    ESCAPE = 'ESCAPE'
    COMMENT_DELIMITER = 'COMMENT_DELIMITER'
    ESCAPE_DELIMITER = 'ESCAPE_DELIMITER'
    ESCAPE_DELIMITER_NAME = 'ESCAPE_DELIMITER_NAME'
    INSTRUCTION_DELIMITER = 'INSTRUCTION_DELIMITER'
    INSTRUCTION_DELIMITER_NAME = 'INSTRUCTION_DELIMITER_NAME'
    INSTRUCTION_DELIMITER_ARGS = 'INSTRUCTION_DELIMITER_ARGS'
    DATA_DELIMITER = 'DATA_DELIMITER'
    DATA_DELIMITER_NAME = 'DATA_DELIMITER_NAME'
    DATA_DELIMITER_ARGS = 'DATA_DELIMITER_ARGS'
    OBJECT_DELIMITER = 'OBJECT_DELIMITER'
    ARRAY_DELIMITER = 'ARRAY_DELIMITER'
    VOID_DELIMITER = 'VOID_DELIMITER'
    PART_DELIMITER = 'PART_DELIMITER'
    DATA = 'DATA'
    GO = 'GO'
    STOP = 'STOP'
    LOCKED = 'LOCKED'

class EmittableEvents(TypedDict):
    content: bool
    end: bool
    endData: bool

class EventListeners(TypedDict):
    content: List[ASLANEventHandler]
    end: List[ASLANEventHandler]
    endData: List[Callable[[ASLANEndDataInstruction], None]]

class ASLANParserSettings(TypedDict):
    prefix: str
    defaultFieldName: str
    eventListeners: EventListeners
    strictStart: bool
    strictEnd: bool
    emittableEvents: EmittableEvents
    multiAslanOutput: bool
    collapseObjectStartWhitespace: bool
    appendSeparator: str

class ASLANDataInsertionType(Enum):
    DEFAULT = 'DEFAULT'
    APPEND = 'APPEND'
    KEEP_FIRST = 'KEEP_FIRST'
    KEEP_LAST = 'KEEP_LAST'

def data_insertion_type_to_string(type: ASLANDataInsertionType) -> str:
    return type.value

def delimiter_type_to_string(type: Optional[ASLANDelimiterType]) -> str:
    if type is None:
        return ''
    return type.value

class ASLANRegisteredInstruction(TypedDict):
    key: Union[str, int]
    name: str
    index: int
    args: List[str]
    partIndex: int

class ASLANParserStateStackFrame(TypedDict):
    innerResult: Union[ASLANObject, ASLANArray]
    dataInsertionTypes: Dict[str, ASLANDataInsertionType]
    dataInsertionLocks: Dict[str, bool]
    currentKey: Union[str, int]
    minArrayIndex: int
    voidFields: Dict[str, bool]
    alreadySeenDuplicateKeys: Dict[str, bool]
    implicitArrays: Dict[str, bool]
    registeredInstructions: List[ASLANRegisteredInstruction]

def create_default_parser_settings() -> ASLANParserSettings:
    return {
        'prefix': 'aslan',
        'defaultFieldName': '_default',
        'eventListeners': {
            'content': [],
            'end': [],
            'endData': []
        },
        'strictStart': False,
        'strictEnd': False,
        'emittableEvents': {
            'content': True,
            'end': True,
            'endData': True
        },
        'multiAslanOutput': False,
        'collapseObjectStartWhitespace': True,
        'appendSeparator': '',
    }

class ASLANParser:
    def __init__(self, parser_settings: dict = None):
        if not parser_settings:
            parser_settings = {}
            
        self.state = ASLANParserState.START
        self.result = {
            '_default': ''
        }
        self.data_insertion_types = {
            '_default': ASLANDataInsertionType.DEFAULT
        }
        self.data_insertion_locks = {
            '_default': False
        }
        self.stack = [{
            'innerResult': self.result,
            'dataInsertionTypes': self.data_insertion_types,
            'dataInsertionLocks': self.data_insertion_locks,
            'currentKey': '_default',
            'minArrayIndex': 0,
            'voidFields': {},
            'alreadySeenDuplicateKeys': {},
            'implicitArrays': {},
            'registeredInstructions': []
        }]
        self.current_delimiter = None
        self.current_value = ''
        self.delimiter_buffer = ''
        self.recent_delimiters = RecentItems[ASLANDelimiterType]()
        self.current_escape_delimiter = None
        self.parsing_locked = False
        self.parser_settings = {
            **create_default_parser_settings(),
            **parser_settings
        }
        self.multi_aslan_results = []
        self.did_stop = True
        self.listener_idempotency_keys = {}
        
        self.delimiter_open_substring = '[' + self.parser_settings['prefix']
        self.stack[0]['currentKey'] = self.parser_settings['defaultFieldName']
        
        if self.parser_settings['strictStart']:
            self.parsing_locked = True
            self.state = ASLANParserState.LOCKED
            
        if self.parser_settings['defaultFieldName'] != '_default':
            self.result = {
                self.parser_settings['defaultFieldName']: ''
            }
            self.data_insertion_types = {
                self.parser_settings['defaultFieldName']: ASLANDataInsertionType.DEFAULT
            }
            self.data_insertion_locks = {
                self.parser_settings['defaultFieldName']: False
            }
            self.stack[0]['currentKey'] = self.parser_settings['defaultFieldName']
            self.stack[0]['innerResult'] = self.result
            self.stack[0]['dataInsertionTypes'] = self.data_insertion_types
            self.stack[0]['dataInsertionLocks'] = self.data_insertion_locks
            
        self.multi_aslan_results.append(self.stack[0]['innerResult'])


    def parse(self, input_str: str) -> Union[ASLANObject, List[Union[ASLANObject, ASLANArray]]]:
        for char in input_str:
            self.handle_next_char(char)
        self.close()
        if self.parser_settings['multiAslanOutput']:
            return self.multi_aslan_results
        return self.stack[0]['innerResult']

    def parse_next(self, input_str: str) -> None:
        for char in input_str:
            self.handle_next_char(char)

    def get_current_value(self) -> str:
        return self.current_value

    def exit_delimiter_into_data(self, char: str) -> None:
        self.current_value += self.delimiter_buffer + char
        self.delimiter_buffer = ''
        self.current_delimiter = None
        self.state = ASLANParserState.DATA

    def handle_next_char(self, char: str) -> None:
        if self.state == ASLANParserState.GO_DELIMITER:
            self.handle_go_delimiter(char)
        elif self.state == ASLANParserState.STOP_DELIMITER:
            self.handle_stop_delimiter(char)
        elif self.state == ASLANParserState.GO:
            self.handle_go(char)
        elif self.state == ASLANParserState.STOP:
            self.handle_stop(char)
        elif self.state == ASLANParserState.START:
            self.handle_start(char)
        elif self.state == ASLANParserState.MAYBE_DELIMITER:
            self.handle_maybe_delimiter(char)
        elif self.state == ASLANParserState.DELIMITER:
            self.handle_delimiter(char)
        elif self.state == ASLANParserState.RESERVED_DELIMITER:
            self.handle_reserved_delimiter(char)
        elif self.state == ASLANParserState.OBJECT:
            self.handle_object(char)
        elif self.state == ASLANParserState.ARRAY:
            self.handle_array(char)
        elif self.state == ASLANParserState.COMMENT:
            self.handle_comment(char)
        elif self.state == ASLANParserState.ESCAPE:
            self.handle_escape(char)
        elif self.state == ASLANParserState.INSTRUCTION_DELIMITER:
            self.handle_instruction_delimiter(char)
        elif self.state == ASLANParserState.INSTRUCTION_DELIMITER_NAME:
            self.handle_instruction_delimiter_name(char)
        elif self.state == ASLANParserState.INSTRUCTION_DELIMITER_ARGS:
            self.handle_instruction_delimiter_args(char)
        elif self.state == ASLANParserState.DATA_DELIMITER:
            self.handle_data_delimiter(char)
        elif self.state == ASLANParserState.DATA_DELIMITER_NAME:
            self.handle_data_delimiter_name(char)
        elif self.state == ASLANParserState.DATA_DELIMITER_ARGS:
            self.handle_data_delimiter_args(char)
        elif self.state == ASLANParserState.OBJECT_DELIMITER:
            self.handle_object_delimiter(char)
        elif self.state == ASLANParserState.ARRAY_DELIMITER:
            self.handle_array_delimiter(char)
        elif self.state == ASLANParserState.VOID_DELIMITER:
            self.handle_void_delimiter(char)
        elif self.state == ASLANParserState.COMMENT_DELIMITER:
            self.handle_comment_delimiter(char)
        elif self.state == ASLANParserState.ESCAPE_DELIMITER:
            self.handle_escape_delimiter(char)
        elif self.state == ASLANParserState.ESCAPE_DELIMITER_NAME:
            self.handle_escape_delimiter_name(char)
        elif self.state == ASLANParserState.PART_DELIMITER:
            self.handle_part_delimiter(char)
        elif self.state == ASLANParserState.DATA:
            self.handle_data(char)
        elif self.state == ASLANParserState.LOCKED:
            self.handle_locked(char)

    def handle_locked(self, char: str) -> None:
        if char == '[':
            self.state = ASLANParserState.MAYBE_DELIMITER
            self.delimiter_buffer = char

    def handle_go_delimiter(self, char: str) -> None:
        if char == ']':
            # Spec: Go delimiters have no <CONTENT> or args
            # VALID GO DELIMITER
            self.state = ASLANParserState.GO
            self.delimiter_buffer = ''
            self.current_value = ''
            self.parsing_locked = False
            if self.parser_settings['strictStart'] and not self.did_stop:
                self.close()
                self.reset()
                self.multi_aslan_results.append(self.stack[0]['innerResult'])
            self.did_stop = False
            return
        # Spec: Go delimiters have no <CONTENT> or args
        # INVALID GO DELIMITER
        self.exit_delimiter_into_data(char)

    def handle_stop_delimiter(self, char: str) -> None:
        if char == ']':
            # Spec: Stop delimiters have no <CONTENT> or args
            # VALID STOP DELIMITER
            self.state = ASLANParserState.STOP
            self.delimiter_buffer = ''
            self.current_value = ''
            if self.parser_settings['strictEnd']:
                if self.parser_settings['strictStart']:
                    self.parsing_locked = True
                self.close()
                self.reset()
                self.multi_aslan_results.append(self.stack[0]['innerResult'])
                self.state = ASLANParserState.START
                self.did_stop = True
            return
        # Spec: Stop delimiters have no <CONTENT> or args
        # INVALID STOP DELIMITER
        self.exit_delimiter_into_data(char)

    def handle_start(self, char: str) -> None:
        if char == '[':
            self.state = ASLANParserState.MAYBE_DELIMITER
            self.delimiter_buffer += char
        else:
            self.state = ASLANParserState.DATA
            self.current_value += char

    def handle_maybe_delimiter(self, char: str) -> None:
        if len(self.delimiter_buffer) > len(self.delimiter_open_substring):
            self.state = ASLANParserState.DATA
            self.current_value += char
            return
        if char == self.delimiter_open_substring[len(self.delimiter_buffer)]:
            self.delimiter_buffer += char
            if self.delimiter_buffer == self.delimiter_open_substring:
                self.state = ASLANParserState.DELIMITER
            return
        self.exit_delimiter_into_data(char)

    def handle_delimiter(self, char: str) -> None:
        if (self.parsing_locked and 
            char != 'g' and 
            not self.parser_settings['strictStart']):
            self.state = ASLANParserState.LOCKED
            return

        self.current_delimiter = {
            'prefix': self.parser_settings['prefix'],
            'suffix': None,
            'content': None,
            'args': []
        }

        if char == 'd':
            self.state = ASLANParserState.DATA_DELIMITER
            self.current_delimiter['suffix'] = ASLANDelimiterType.DATA
            self.delimiter_buffer += char
        elif char == 'o':
            self.state = ASLANParserState.OBJECT_DELIMITER
            self.current_delimiter['suffix'] = ASLANDelimiterType.OBJECT
            self.delimiter_buffer += char
        elif char == 'i':
            self.state = ASLANParserState.INSTRUCTION_DELIMITER
            self.current_delimiter['suffix'] = ASLANDelimiterType.INSTRUCTION
            self.delimiter_buffer += char
        elif char == 'a':
            self.state = ASLANParserState.ARRAY_DELIMITER
            self.current_delimiter['suffix'] = ASLANDelimiterType.ARRAY
            self.delimiter_buffer += char
        elif char == 'c':
            self.state = ASLANParserState.COMMENT_DELIMITER
            self.current_delimiter['suffix'] = ASLANDelimiterType.COMMENT
            self.delimiter_buffer += char
        elif char == 'e':
            self.state = ASLANParserState.ESCAPE_DELIMITER
            self.current_delimiter['suffix'] = ASLANDelimiterType.ESCAPE
            self.delimiter_buffer += char
        elif char == 'p':
            self.state = ASLANParserState.PART_DELIMITER
            self.current_delimiter['suffix'] = ASLANDelimiterType.PART
            self.delimiter_buffer += char
        elif char == 'v':
            self.state = ASLANParserState.VOID_DELIMITER
            self.current_delimiter['suffix'] = ASLANDelimiterType.VOID
            self.delimiter_buffer += char
        elif char == 'g':
            self.state = ASLANParserState.GO_DELIMITER
            self.current_delimiter['suffix'] = ASLANDelimiterType.GO
            self.delimiter_buffer += char
        elif char == 's':
            self.state = ASLANParserState.STOP_DELIMITER
            self.current_delimiter['suffix'] = ASLANDelimiterType.STOP
            self.delimiter_buffer += char
        else:
            if char.isalnum():
                self.state = ASLANParserState.RESERVED_DELIMITER
                self.delimiter_buffer += char
                return
            self.exit_delimiter_into_data(char)
            return

        if self.current_delimiter['suffix'] is not None:
            self.recent_delimiters.add(self.current_delimiter['suffix'])

    def handle_reserved_delimiter(self, char: str) -> None:
        if self.parsing_locked:
            self.state = ASLANParserState.LOCKED
            return
        if char != ']':
            # Spec: Reserved delimiters contain no <CONTENT> or args
            # INVALID RESERVED DELIMITER
            return self.exit_delimiter_into_data(char)
        self.delimiter_buffer = ''
        self.state = ASLANParserState.DATA
        self.current_value = ''

    def handle_object_delimiter(self, char: str) -> None:
        if self.parsing_locked:
            self.state = ASLANParserState.LOCKED
            return
        if self.current_escape_delimiter:
            return self.exit_delimiter_into_data(char)
        if char == ']':
            # Spec: Object delimiters have no <CONTENT> or args
            # VALID OBJECT DELIMITER
            self.state = ASLANParserState.OBJECT
            self.delimiter_buffer = ''
            second_most_recent_material_delimiter = self.get_2nd_most_recent_material_delimiter()
            if (self.get_object_safe_latest_result() or 
                second_most_recent_material_delimiter != ASLANDelimiterType.DATA):
                if (self.get_current_key() not in self.get_latest_result() or
                    not isinstance(self.get_latest_result()[self.get_current_key()], dict) or
                    second_most_recent_material_delimiter != ASLANDelimiterType.DATA):
                    if self.stack[-1]['alreadySeenDuplicateKeys'].get(self.get_current_key()):
                        self.stack[-1]['alreadySeenDuplicateKeys'][self.get_current_key()] = False
                        self.create_new_object()
                        return
                    if len(self.stack) > 1:
                        self.emit_end_events_if_required()
                        self.emit_end_data_events_if_required()
                        self.stack.pop()
                else:
                    self.create_new_object()
                return
            self.create_new_object()
            return
        # Spec: Object delimiters have no <CONTENT> or args
        # INVALID OBJECT DELIMITER
        self.exit_delimiter_into_data(char)

    def get_object_safe_latest_result(self):
        if isinstance(self.get_latest_result(), list):
            if self.get_current_key() >= len(self.get_latest_result()):
                return None
            else: 
                latest_result = self.get_latest_result()[self.get_current_key()]
                if isinstance(latest_result, str):
                    if self.parser_settings['collapseObjectStartWhitespace']:
                        return latest_result.strip()
                    return latest_result
                return latest_result
        if self.get_current_key() not in self.get_latest_result():
            return None
        latest_result = self.get_latest_result()[self.get_current_key()]
        if isinstance(latest_result, str):
            if self.parser_settings['collapseObjectStartWhitespace']:
                return latest_result.strip()
            return latest_result
        return latest_result

    def create_new_object(self) -> None:
        self.current_value = ''
        if isinstance(self.get_latest_result(), list):
            while len(self.get_latest_result()) <= self.get_current_key():
                self.get_latest_result().append(None)
            self.get_latest_result()[self.get_current_key()] = {}
        else:
            self.get_latest_result()[self.get_current_key()] = {}

        self.stack.append({
            'innerResult': self.get_latest_result()[self.get_current_key()],
            'dataInsertionTypes': {},
            'dataInsertionLocks': {},
            'currentKey': self.parser_settings['defaultFieldName'],
            'minArrayIndex': 0,
            'voidFields': {},
            'alreadySeenDuplicateKeys': {},
            'implicitArrays': {},
            'registeredInstructions': []
        })

    def handle_instruction_delimiter(self, char: str) -> None:
        if self.parsing_locked:
            self.state = ASLANParserState.LOCKED
            return
        if self.current_escape_delimiter:
            return self.exit_delimiter_into_data(char)
        if char == ']':
            # Spec: Instruction delimiters must contain <CONTENT>
            # INVALID INSTRUCTION DELIMITER
            return self.exit_delimiter_into_data(char)
        if char == '_':
            # Spec: Instruction delimiters must contain <CONTENT>
            self.state = ASLANParserState.INSTRUCTION_DELIMITER_NAME
            self.delimiter_buffer += char
            self.current_delimiter['content'] = ''
            self.current_value = ''
            return
        # Spec: Instruction delimiters must contain <CONTENT>
        # INVALID INSTRUCTION DELIMITER
        self.exit_delimiter_into_data(char)

    def handle_instruction_delimiter_name(self, char: str) -> None:
        if self.parsing_locked:
            self.state = ASLANParserState.LOCKED
            return
        if self.current_delimiter['content'] != '' and char == ':':
            if self.current_delimiter['content'].endswith('_'):
                # Spec: Delimiter <CONTENT> may not end with an underscore.
                # INVALID INSTRUCTION DELIMITER
                return self.exit_delimiter_into_data(char)
            # Spec: Instructions may have arguments.
            self.state = ASLANParserState.INSTRUCTION_DELIMITER_ARGS
            self.current_delimiter['args'] = ['']
            self.current_value = ''
            self.delimiter_buffer += char
            return
        if char == '_' and self.current_delimiter['content'] == '':
            # Spec: Delimiter <CONTENT> may not start with an underscore.
            # INVALID INSTRUCTION DELIMITER
            return self.exit_delimiter_into_data(char)
        if char == ']':
            if self.current_delimiter['content'].endswith('_'):
                # Spec: Delimiter <CONTENT> may not end with an underscore.
                # INVALID INSTRUCTION DELIMITER
                return self.exit_delimiter_into_data(char)
            # Spec: Instruction delimiter of the form [<PREFIX>i_<CONTENT>]
            # VALID INSTRUCTION DELIMITER
            index = 0
            part_index = 0
            if isinstance(self.get_latest_result()[self.get_current_key()], list):
                array = self.get_latest_result()[self.get_current_key()]
                index = len(array[len(array)-1])
                part_index = len(array)-1
            elif isinstance(self.get_latest_result()[self.get_current_key()], str):
                index = len(self.get_latest_result()[self.get_current_key()])
            self.state = ASLANParserState.DATA
            if (not self.stack[len(self.stack)-1]['alreadySeenDuplicateKeys'].get(self.get_current_key()) or
                    self.data_insertion_types[self.get_current_key()] != ASLANDataInsertionType.KEEP_FIRST):
                self.register_instruction({
                    'name': self.current_delimiter['content'] or '',
                    'index': index,
                    'args': self.current_delimiter['args'],
                    'key': self.get_current_key(),
                    'partIndex': part_index
                })
                if not isinstance(self.get_latest_result()[self.get_current_key()], dict):
                    self.emit_content_events_for_primitive()
                if self.stack[len(self.stack)-1]['implicitArrays'].get(self.get_current_key()):
                    self.emit_content_events_for_implicit_array()
            self.delimiter_buffer = ''
            self.current_value = ''
            return
        if not re.match(r'^[a-zA-Z0-9_]$', char):
            # Spec: Delimiter <CONTENT> may only contain alphanumeric characters and underscores.
            # INVALID INSTRUCTION DELIMITER
            return self.exit_delimiter_into_data(char)
        self.current_delimiter['content'] += char
        self.delimiter_buffer += char

    def handle_instruction_delimiter_args(self, char: str) -> None:
        if self.parsing_locked:
            self.state = ASLANParserState.LOCKED
            return
        if char == ']':
            # Spec: Instruction delimiter of the form [<PREFIX>i_<CONTENT>:<ARG0>:<ARG1>:<ARG2>:...]
            # VALID INSTRUCTION DELIMITER
            index = 0
            part_index = 0
            if isinstance(self.get_latest_result()[self.get_current_key()], list):
                array = self.get_latest_result()[self.get_current_key()]
                index = len(array[len(array)-1])
                part_index = len(array)-1
            elif isinstance(self.get_latest_result()[self.get_current_key()], str):
                index = len(self.get_latest_result()[self.get_current_key()])
            self.state = ASLANParserState.DATA
            self.delimiter_buffer = ''
            self.current_value = ''
            if (not self.stack[len(self.stack)-1]['alreadySeenDuplicateKeys'].get(self.get_current_key()) or
                    self.data_insertion_types[self.get_current_key()] != ASLANDataInsertionType.KEEP_FIRST):
                self.register_instruction({
                    'name': self.current_delimiter['content'] or '',
                    'index': index,
                    'args': self.current_delimiter['args'],
                    'key': self.get_current_key(),
                    'partIndex': part_index
                })
                if not isinstance(self.get_latest_result()[self.get_current_key()], dict):
                    self.emit_content_events_for_primitive()
                if self.stack[len(self.stack)-1]['implicitArrays'].get(self.get_current_key()):
                    self.emit_content_events_for_implicit_array()
            return
        if char == ':':
            # Start a new arg
            self.delimiter_buffer += char
            self.current_delimiter['args'].append('')
            return
        # Add to the current arg
        self.current_delimiter['args'][len(self.current_delimiter['args'])-1] += char
        self.delimiter_buffer += char

    def register_instruction(self, instruction: dict) -> None:
        self.stack[len(self.stack)-1]['registeredInstructions'].append(instruction)

    def handle_data_delimiter(self, char: str) -> None:
        if self.parsing_locked:
            self.state = ASLANParserState.LOCKED
            return
        if self.current_escape_delimiter:
            return self.exit_delimiter_into_data(char)
        if char == ']':
            latest_result = self.get_latest_result()
            if isinstance(latest_result, list):
                # Spec: Data delimiters can have no <CONTENT> or args if the current result is an array.
                # VALID DATA DELIMITER
                self.state = ASLANParserState.DATA
                self.delimiter_buffer = ''
                self.current_value = ''
                self.emit_end_events_if_required()
                self.emit_end_data_events_if_required()
                self.next_key()
                return
            # Spec: Data delimiters must contain <CONTENT> if the current result is not an array.
            # INVALID DATA DELIMITER
            return self.exit_delimiter_into_data(char)
        if char == '_':
            self.state = ASLANParserState.DATA_DELIMITER_NAME
            self.delimiter_buffer += char
            self.current_delimiter['content'] = ''
            self.current_value = ''
            return
        # Spec: Data delimiters must be valid of the form [<PREFIX>d_<CONTENT>] or [<PREFIX>d_<CONTENT>:<ARG0>:<ARG1>:<ARG2>:...]
        # INVALID DATA DELIMITER
        self.exit_delimiter_into_data(char)

    def handle_data_delimiter_name(self, char: str) -> None:
        if self.parsing_locked:
            self.state = ASLANParserState.LOCKED
            return
        if self.current_delimiter['content'] != '' and char == ':':
            if self.current_delimiter['content'].endswith('_'):
                # Spec: Delimiter <CONTENT> may not end with an underscore.
                # INVALID DATA DELIMITER
                return self.exit_delimiter_into_data(char)
            # Spec: Data may have arguments.
            self.state = ASLANParserState.DATA_DELIMITER_ARGS
            self.current_delimiter['args'] = ['']
            self.current_value = ''
            self.delimiter_buffer += char
            self.emit_end_events_if_required()
            self.emit_end_data_events_if_required()
            self.next_key()
            return
        if char == '_' and self.current_delimiter['content'] == '':
            # Spec: Delimiter <CONTENT> may not start with an underscore.
            # INVALID DATA DELIMITER
            return self.exit_delimiter_into_data(char)
        if char == ']':
            if self.current_delimiter['content'].endswith('_'):
                # Spec: Delimiter <CONTENT> may not end with an underscore.
                # INVALID DATA DELIMITER
                return self.exit_delimiter_into_data(char)
            # Spec: Data delimiter of the form [<PREFIX>d_<CONTENT>]
            # VALID DATA DELIMITER
            self.state = ASLANParserState.DATA
            self.emit_end_events_if_required()
            self.emit_end_data_events_if_required()
            self.next_key()
            self.delimiter_buffer = ''
            self.set_data_insertion_type(ASLANDataInsertionType.DEFAULT)
            if (self.stack[len(self.stack)-1]['alreadySeenDuplicateKeys'].get(self.get_current_key()) and
                self.get_latest_result().get(self.get_current_key()) and
                not isinstance(self.get_latest_result()[self.get_current_key()], dict)):
                self.current_value = self.parser_settings['appendSeparator']
                self.store_current_value()
            self.current_value = ''
            return
        if not re.match(r'^[a-zA-Z0-9_]$', char):
            # Spec: Delimiter <CONTENT> may only contain alphanumeric characters and underscores.
            # INVALID DATA DELIMITER
            return self.exit_delimiter_into_data(char)
        self.current_delimiter['content'] += char
        self.delimiter_buffer += char

    def handle_data_delimiter_args(self, char: str) -> None:
        if self.parsing_locked:
            self.state = ASLANParserState.LOCKED
            return
        if char == ']':
            # Spec: Data delimiter of the form [<PREFIX>d_<CONTENT>:<ARG0>:<ARG1>:<ARG2>:...]
            # VALID DATA DELIMITER
            self.state = ASLANParserState.DATA
            self.delimiter_buffer = ''
            arg = self.current_delimiter['args'][0]
            if arg == 'a':
                self.set_data_insertion_type(ASLANDataInsertionType.APPEND)
            elif arg == 'f':
                self.set_data_insertion_type(ASLANDataInsertionType.KEEP_FIRST)
            elif arg == 'l':
                self.set_data_insertion_type(ASLANDataInsertionType.KEEP_LAST)
            else:
                self.set_data_insertion_type(ASLANDataInsertionType.DEFAULT)
            self.emit_end_events_if_required()
            self.emit_end_data_events_if_required()
            if (self.stack[len(self.stack)-1]['alreadySeenDuplicateKeys'].get(self.get_current_key()) and
                (self.stack[len(self.stack)-1]['dataInsertionTypes'].get(self.get_current_key()) == ASLANDataInsertionType.APPEND or
                 self.stack[len(self.stack)-1]['dataInsertionTypes'].get(self.get_current_key()) == ASLANDataInsertionType.DEFAULT) and
                self.get_latest_result().get(self.get_current_key()) and
                not isinstance(self.get_latest_result()[self.get_current_key()], dict)):
                self.current_value = self.parser_settings['appendSeparator']
                self.store_current_value()
            self.current_value = ''
            return
        if char == ':':
            # Start a new arg
            self.delimiter_buffer += char
            self.current_delimiter['args'].append('')
            return
        # Add to the current arg
        self.current_delimiter['args'][len(self.current_delimiter['args'])-1] += char
        self.delimiter_buffer += char

    def handle_array_delimiter(self, char: str) -> None:
        if self.parsing_locked:
            self.state = ASLANParserState.LOCKED
            return
        if self.current_escape_delimiter:
            return self.exit_delimiter_into_data(char)
        if char == ']':
            # Spec: Array delimiters have no <CONTENT> or args
            # VALID ARRAY DELIMITER
            self.state = ASLANParserState.ARRAY
            self.delimiter_buffer = ''
            second_most_recent_material_delimiter = self.get_2nd_most_recent_material_delimiter()
            if (self.get_object_safe_latest_result() or
                second_most_recent_material_delimiter != ASLANDelimiterType.DATA):
                if (self.get_current_key() not in self.get_latest_result() or
                    not isinstance(self.get_latest_result()[self.get_current_key()], dict) or
                    second_most_recent_material_delimiter != ASLANDelimiterType.DATA):
                    if self.stack[len(self.stack)-1]['alreadySeenDuplicateKeys'].get(self.get_current_key()):
                        self.stack[len(self.stack)-1]['alreadySeenDuplicateKeys'][self.get_current_key()] = False
                        self.create_new_array()
                        return
                    if len(self.stack) > 1:
                        self.emit_end_events_if_required()
                        self.emit_end_data_events_if_required()
                        print('popping')
                        self.stack.pop()
                else:
                    self.create_new_array()
                return
            self.create_new_array()
            return
        # Spec: Array delimiters have no <CONTENT> or args
        # INVALID ARRAY DELIMITER
        self.exit_delimiter_into_data(char)

    def create_new_array(self) -> None:
        self.current_value = ''
        if isinstance(self.get_latest_result(), list):
            while len(self.get_latest_result()) <= self.get_current_key():
                self.get_latest_result().append(None)
            self.get_latest_result()[self.get_current_key()] = []
        else:
            self.get_latest_result()[self.get_current_key()] = []

        self.stack.append({
            'innerResult': self.get_latest_result()[self.get_current_key()],
            'dataInsertionTypes': {},
            'dataInsertionLocks': {},
            'currentKey': -1,
            'minArrayIndex': 0,
            'voidFields': {},
            'alreadySeenDuplicateKeys': {},
            'implicitArrays': {},
            'registeredInstructions': []
        })
    
    def handle_void_delimiter(self, char: str) -> None:
        if self.parsing_locked:
            self.state = ASLANParserState.LOCKED
            return
        if self.current_escape_delimiter:
            return self.exit_delimiter_into_data(char)
        if char == ']':
            # Spec: Void delimiters have no <CONTENT> or args
            # VALID VOID DELIMITER
            self.state = ASLANParserState.DATA
            self.delimiter_buffer = ''
            self.current_value = ''
            self.stack[len(self.stack)-1]['voidFields'][self.get_current_key()] = True
            return
        # Spec: Void delimiters have no <CONTENT> or args
        # INVALID VOID DELIMITER
        self.exit_delimiter_into_data(char)

    def handle_comment_delimiter(self, char: str) -> None:
        if self.parsing_locked:
            self.state = ASLANParserState.LOCKED
            return
        if self.current_escape_delimiter:
            return self.exit_delimiter_into_data(char)
        if char == ']':
            # Spec: Comment delimiters have no <CONTENT> or args
            # VALID COMMENT DELIMITER
            self.state = ASLANParserState.COMMENT
            self.delimiter_buffer = ''
            self.current_value = ''
            return
        # Spec: Comment delimiters have no <CONTENT> or args
        # INVALID COMMENT DELIMITER
        self.exit_delimiter_into_data(char)

    def handle_escape_delimiter(self, char: str) -> None:
        if self.parsing_locked:
            self.state = ASLANParserState.LOCKED
            return
        if char == ']':
            # Spec: Escape delimiters must contain <CONTENT>
            # INVALID ESCAPE DELIMITER
            return self.exit_delimiter_into_data(char)
        if char == '_':
            self.state = ASLANParserState.ESCAPE_DELIMITER_NAME
            self.delimiter_buffer += char
            self.current_delimiter['content'] = ''
            self.current_value = ''
            return
        # Spec: Escape delimiters must be valid of the form [<PREFIX>e_<CONTENT>]
        # INVALID ESCAPE DELIMITER
        self.exit_delimiter_into_data(char)

    def handle_escape_delimiter_name(self, char: str) -> None:
        if self.parsing_locked:
            self.state = ASLANParserState.LOCKED
            return
        if char == '_' and self.current_delimiter['content'] == '':
            # Spec: Delimiter <CONTENT> may not start with an underscore.
            # INVALID ESCAPE DELIMITER
            return self.exit_delimiter_into_data(char)
        if char == ']':
            if self.current_delimiter['content'].endswith('_'):
                # Spec: Delimiter <CONTENT> may not end with an underscore.
                # INVALID ESCAPE DELIMITER
                return self.exit_delimiter_into_data(char)
            # Spec: Escape delimiter of the form [<PREFIX>e_<CONTENT>]
            # VALID ESCAPE DELIMITER
            self.state = ASLANParserState.ESCAPE
            self.delimiter_buffer = ''
            self.current_value = ''
            if not self.current_escape_delimiter:
                self.current_escape_delimiter = self.current_delimiter['content']
            elif self.current_escape_delimiter != self.current_delimiter['content']:
                # Make sure we write out the escape delimiter with different content since the escape hasn't closed
                self.current_value = f"[{self.current_delimiter['prefix']}e_{self.current_delimiter['content']}"
                self.store_current_value()
                # Spec: Escape delimiters must be the same for the entire string.
                # INVALID ESCAPE DELIMITER
                return self.exit_delimiter_into_data(char)
            else:
                self.current_escape_delimiter = None
                self.state = ASLANParserState.DATA
                self.delimiter_buffer = ''
                self.current_value = ''
            return
        if not re.match(r'^[a-zA-Z0-9_]$', char):
            # Spec: Delimiter <CONTENT> may only contain alphanumeric characters and underscores.
            # INVALID ESCAPE DELIMITER
            return self.exit_delimiter_into_data(char)
        self.current_delimiter['content'] += char
        self.delimiter_buffer += char

    def handle_part_delimiter(self, char: str) -> None:
        if self.parsing_locked:
            self.state = ASLANParserState.LOCKED
            return
        if self.current_escape_delimiter:
            return self.exit_delimiter_into_data(char)
        if char == ']':
            # Spec: Part delimiters have no <CONTENT> or args
            # VALID PART DELIMITER
            if not self.data_insertion_locks.get(self.get_current_key()):
                if not self.get_latest_result().get(self.get_current_key()):
                    self.stack[len(self.stack)-1]['implicitArrays'][self.get_current_key()] = True
                    self.get_latest_result()[self.get_current_key()] = ['']
                elif isinstance(self.get_latest_result()[self.get_current_key()], str):
                    self.stack[len(self.stack)-1]['implicitArrays'][self.get_current_key()] = True
                    self.get_latest_result()[self.get_current_key()] = [
                        self.get_latest_result()[self.get_current_key()]
                    ]
                    self.get_latest_result()[self.get_current_key()].append('')
                else:
                    self.emit_end_events_if_required()
                    self.get_latest_result()[self.get_current_key()].append('')
            self.state = ASLANParserState.DATA
            self.delimiter_buffer = ''
            self.current_value = ''
            self.next_key()
            return
        # Spec: Part delimiters have no <CONTENT> or args
        # INVALID PART DELIMITER
        self.exit_delimiter_into_data(char)

    def handle_go(self, char: str) -> None:
        if char == '[':
            self.state = ASLANParserState.MAYBE_DELIMITER
            self.delimiter_buffer += char
            return
        self.exit_delimiter_into_data(char)

    def handle_stop(self, char: str) -> None:
        if self.parsing_locked:
            self.state = ASLANParserState.LOCKED
            return
        if char == '[':
            self.state = ASLANParserState.MAYBE_DELIMITER
            self.delimiter_buffer += char
            return
        self.append_to_current_value(char)

    def handle_object(self, char: str) -> None:
        if self.parsing_locked:
            self.state = ASLANParserState.LOCKED
            return
        if char == '[':
            self.state = ASLANParserState.MAYBE_DELIMITER
            self.delimiter_buffer += char
            return
        self.append_to_current_value(char)

    def handle_array(self, char: str) -> None:
        if self.parsing_locked:
            self.state = ASLANParserState.LOCKED
            return
        if char == '[':
            self.state = ASLANParserState.MAYBE_DELIMITER
            self.delimiter_buffer += char
            return
        self.append_to_current_value(char)

    def handle_comment(self, char: str) -> None:
        if self.parsing_locked:
            self.state = ASLANParserState.LOCKED
            return
        if char == '[':
            self.state = ASLANParserState.MAYBE_DELIMITER
            self.delimiter_buffer += char

    def handle_escape(self, char: str) -> None:
        if self.parsing_locked:
            self.state = ASLANParserState.LOCKED
            return
        if char == '[':
            self.state = ASLANParserState.MAYBE_DELIMITER
            self.delimiter_buffer += char
        self.append_to_current_value(char)
        self.store_current_value()
        self.state = ASLANParserState.DATA
        self.delimiter_buffer = ''
        self.current_value = ''

    def handle_data(self, char: str) -> None:
        if self.parsing_locked:
            self.state = ASLANParserState.LOCKED
            return
        if char == '[':
            self.state = ASLANParserState.MAYBE_DELIMITER
            self.delimiter_buffer += char
            return
        self.append_to_current_value(char)
        self.store_current_value()

    def add_event_listener(self, event: Literal['content', 'end', 'end_data'], 
                         callback: ASLANEventHandler,
                         idempotency_key: str = None) -> ASLANEventHandler:
        if not idempotency_key:
            idempotency_key = generate_random_idempotency_key()
        if self.listener_idempotency_keys.get(idempotency_key):
            return callback
        self.listener_idempotency_keys[idempotency_key] = callback
        event_name = 'endData' if event == 'end_data' else event
        self.parser_settings['eventListeners'][event_name].append(callback)
        return callback

    def remove_event_listener(self, event: Literal['content', 'end', 'end_data'],
                           callback: ASLANEventHandler) -> None:
        event_name = 'endData' if event == 'end_data' else event
        self.parser_settings['eventListeners'][event_name] = [
            listener for listener in self.parser_settings['eventListeners'][event_name]
            if listener != callback
        ]
        for key, value in list(self.listener_idempotency_keys.items()):
            if value == callback:
                del self.listener_idempotency_keys[key]

    def clear_event_listeners(self) -> None:
        self.listener_idempotency_keys = {}
        self.parser_settings['eventListeners'] = {
            'content': [],
            'end': [],
            'end_data': []
        }

    def emit_end_events_if_required(self) -> None:
        if not self.parser_settings['emittableEvents']['end']:
            return
        if self.get_current_key() not in self.get_latest_result():
            return
        if not isinstance(self.get_latest_result()[self.get_current_key()], (dict, list)):
            self.emit_content_events_for_primitive('end')
        if self.stack[len(self.stack)-1]['implicitArrays'].get(self.get_current_key()):
            self.emit_content_events_for_implicit_array('end')

    def emit_end_data_events_if_required(self) -> None:
        if not self.parser_settings['emittableEvents']['endData']:
            return
        if self.get_current_key() not in self.get_latest_result():
            return
        if not isinstance(self.get_latest_result()[self.get_current_key()], (dict, list)):
            content = [{
                'value': self.get_latest_result()[self.get_current_key()],
                'partIndex': 0,
                'instructions': self.stack[len(self.stack)-1]['registeredInstructions']
            }]
            self.emit_end_data_event(content, self.get_current_key(), self.get_current_path())
            return

        if self.stack[len(self.stack)-1]['implicitArrays'].get(self.get_current_key()):
            current_stack_frame = self.stack[len(self.stack)-1]
            content = []
            instructions_by_part_index = {}
            
            for instruction in current_stack_frame['registeredInstructions']:
                if instruction['partIndex'] not in instructions_by_part_index:
                    instructions_by_part_index[instruction['partIndex']] = []
                instructions_by_part_index[instruction['partIndex']].append({
                    'name': instruction['name'],
                    'args': instruction['args'],
                    'index': instruction['index']
                })

            if isinstance(self.get_latest_result()[self.get_current_key()], list):
                for i in range(len(self.get_latest_result()[self.get_current_key()])):
                    content.append({
                        'value': self.get_latest_result()[self.get_current_key()][i],
                        'partIndex': i,
                        'instructions': instructions_by_part_index.get(i, [])
                    })

            self.emit_end_data_event(content, self.get_current_key(), self.get_current_path())

    def emit_content_events_for_primitive(self, tag: str = 'content') -> None:
        if not self.parser_settings['emittableEvents'][tag]:
            return
            
        for instruction in self.stack[-1]['registeredInstructions']:
            if (instruction['key'] != self.get_current_key() or 
                (instruction['key'] == self.get_current_key() and 
                 instruction['partIndex'] != 0)):
                continue
                
            self.emit_content_or_end_event(
                self.get_latest_result()[self.get_current_key()],
                instruction,
                0,
                self.get_current_key(),
                self.get_current_path(),
                tag
            )

    def emit_content_events_for_implicit_array(self, tag: str = 'content') -> None:
        if not self.parser_settings['emittableEvents'][tag]:
            return
            
        for instruction in self.stack[-1]['registeredInstructions']:
            latest_result = self.get_latest_result()[self.get_current_key()]
            if (instruction['key'] != self.get_current_key() or
                (instruction['key'] == self.get_current_key() and
                 instruction['partIndex'] != len(latest_result) - 1)):
                continue
                
            self.emit_content_or_end_event(
                latest_result[len(latest_result) - 1],
                instruction,
                len(latest_result) - 1,
                self.get_current_key(),
                self.get_current_path(),
                tag
            )

    def emit_content_or_end_event(
        self,
        value: str,
        instruction: ASLANRegisteredInstruction,
        part_index: int,
        field_name: Union[str, int],
        path: List[str],
        tag: str = 'content'
    ) -> None:
        for callback in self.parser_settings['eventListeners'][tag]:
            callback({
                'tag': tag,
                'content': value,
                'partIndex': part_index,
                'fieldName': field_name,
                'path': path,
                'structure': self.get_result(),
                'instruction': instruction['name'],
                'args': instruction['args'],
                'index': instruction['index'],
                'multiAslanIndex': len(self.multi_aslan_results) - 1
            })

    def emit_end_data_event(
        self,
        content: List[Dict[str, Union[str, int, List[Dict[str, Union[str, List[str], int]]]]]],
        field_name: Union[str, int],
        path: List[str]
    ) -> None:
        if not self.parser_settings['emittableEvents']['endData']:
            return
            
        for callback in self.parser_settings['eventListeners']['endData']:
            callback({
                'tag': 'end_data',
                'content': content,
                'fieldName': field_name,
                'path': path,
                'structure': self.get_result(),
                'multiAslanIndex': len(self.multi_aslan_results) - 1
            })

    def set_current_value(self, value: str) -> None:
        self.current_value = value

    def append_to_current_value(self, value: str) -> None:
        self.current_value += value

    def store_current_value(self) -> None:
        if self.stack[-1]['voidFields'].get(self.get_current_key()):
            self.current_value = ''
            self.get_latest_result()[self.get_current_key()] = None
            return

        if self.current_value:
            if isinstance(self.get_latest_result(), list):
                if self.get_current_key() >= len(self.get_latest_result()):
                    while len(self.get_latest_result()) <= self.get_current_key():
                        self.get_latest_result().append(None)
            else:
                if self.get_current_key() not in self.get_latest_result():
                    self.get_latest_result()[self.get_current_key()] = ''
                
            if (not self.data_insertion_locks.get(self.get_current_key()) and
                not isinstance(self.get_latest_result()[self.get_current_key()], (dict, list))):
                if isinstance(self.get_latest_result(), list) and self.get_latest_result()[self.get_current_key()] is None:
                    self.get_latest_result()[self.get_current_key()] = ''
                self.get_latest_result()[self.get_current_key()] += self.current_value
                self.emit_content_events_for_primitive()
                
            if (not self.data_insertion_locks.get(self.get_current_key()) and
                self.stack[-1]['implicitArrays'].get(self.get_current_key())):
                latest_result = self.get_latest_result()[self.get_current_key()]
                latest_result[len(latest_result) - 1] += self.current_value
                self.emit_content_events_for_implicit_array()
                
            self.current_value = ''

    def set_data_insertion_type(self, type: ASLANDataInsertionType) -> None:
        # Spec: Data insertion type can only be set once for a given key in an object/array.
        # NOTE: We keep the behavior as defined on the first occurrence of the key to avoid LLM instability causing difficult to predict behavior.
        if self.data_insertion_types.get(self.get_current_key()) is not None:
            # If we're trying to set the type again then we've hit a duplicate key so check if it's a KEEP_LAST and clear the value if so
            # Otherwise if it's KEEP_FIRST lock future appearances of the key.
            current_type = self.data_insertion_types[self.get_current_key()]
            if current_type == ASLANDataInsertionType.KEEP_LAST:
                self.get_latest_result()[self.get_current_key()] = ''
                self.stack[-1]['registeredInstructions'] = [
                    instruction for instruction in self.stack[-1]['registeredInstructions']
                    if instruction['key'] != self.get_current_key()
                ]
            elif current_type == ASLANDataInsertionType.KEEP_FIRST:
                self.data_insertion_locks[self.get_current_key()] = True
            return
            
        self.data_insertion_types[self.get_current_key()] = type

    def next_key(self) -> None:
        is_array = isinstance(self.get_latest_result(), list)
        if is_array:
            if self.current_delimiter and self.current_delimiter.get('content'):
                # Explicit index
                new_index = int(self.current_delimiter['content']) if self.current_delimiter['content'].isdigit() else None
                if new_index is not None:
                    self.set_current_key(new_index)
                    self.set_min_array_index(
                        max(self.get_min_array_index(), new_index + 1)
                    )
                else:
                    self.set_current_key(self.get_min_array_index())
                    self.set_min_array_index(self.get_min_array_index() + 1)
            else:
                # Implicit index
                self.set_current_key(self.get_min_array_index())
                self.set_min_array_index(self.get_min_array_index() + 1)
        else:
            # Object
            if self.current_delimiter and self.current_delimiter.get('content'):
                if (self.parser_settings['defaultFieldName'] in self.get_latest_result() and
                    self.get_latest_result()[self.parser_settings['defaultFieldName']] == ''):
                    self.get_latest_result()[self.parser_settings['defaultFieldName']] = None
                
                self.set_current_key(self.current_delimiter['content'])
                if self.get_current_key() in self.get_latest_result():
                    self.stack[-1]['alreadySeenDuplicateKeys'][
                        self.current_delimiter['content']
                    ] = True
                else:
                    self.get_latest_result()[self.current_delimiter['content']] = ''

    def get_current_path(self) -> list[str]:
        path = []
        for stack_frame in self.stack:
            current_key = stack_frame['currentKey']
            if (isinstance(current_key, str) and 
                current_key != self.parser_settings['defaultFieldName']):
                path.append(current_key)
            elif isinstance(current_key, int):
                path.append(str(current_key))
        return path

    def close(self) -> None:
        self.emit_end_events_if_required()
        self.emit_end_data_events_if_required() 
        self.store_current_value()

    def get_result(self):
        return self.stack[0]['innerResult']

    def get_results(self):
        return self.multi_aslan_results

    def reset(self) -> None:
        self.result = {
            self.parser_settings['defaultFieldName']: ''
        }
        self.data_insertion_types = {
            self.parser_settings['defaultFieldName']: ASLANDataInsertionType.DEFAULT
        }
        self.data_insertion_locks = {
            self.parser_settings['defaultFieldName']: False
        }
        self.stack = [{
            'innerResult': self.result,
            'dataInsertionTypes': {},
            'dataInsertionLocks': {},
            'currentKey': self.parser_settings['defaultFieldName'],
            'minArrayIndex': 0,
            'voidFields': {},
            'alreadySeenDuplicateKeys': {},
            'implicitArrays': {},
            'registeredInstructions': []
        }]

    def get_current_key(self):
        return self.stack[-1]['currentKey']

    def set_current_key(self, key):
        self.stack[-1]['currentKey'] = key

    def get_min_array_index(self):
        return self.stack[-1]['minArrayIndex']

    def set_min_array_index(self, index: int):
        self.stack[-1]['minArrayIndex'] = index

    def get_latest_result(self):
        return self.stack[-1]['innerResult']

    def get_2nd_most_recent_material_delimiter(self):
        return self.recent_delimiters.get_nth_most_recent_not_in(
            2,
            {ASLANDelimiterType.COMMENT, ASLANDelimiterType.ESCAPE}
        )