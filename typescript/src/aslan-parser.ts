type ASLANValue = string | ASLANObject | ASLANArray | null;
type ASLANObject = { [key: string]: ASLANValue };
type ASLANArray = ASLANValue[];

type ASLANInstruction = {
  content: string;
  partIndex: number;
  fieldName: string;
  path: string[];
  structure: ASLANObject;
  instruction: string;
  args: string[];
  index: number;
  tag: 'CONTENT' | 'END' | 'END DATA';
};

enum ASLANDelimiterType {
  DATA,
  OBJECT,
  INSTRUCTION,
  ARRAY,
  COMMENT,
  ESCAPE,
  PART,
  VOID,
  GO,
  STOP,
}

type ASLANDuplicateKeyBehavior = 'a' | 'f' | 'l';

type ASLANEventListener = (instruction: ASLANInstruction) => void;

type ASLANDelimiterData = {
  prefix: string | null;
  suffix: ASLANDelimiterType | null;
  content: string | null;
  args: string[];
};

enum ASLANParserState {
  GO_DELIMITER,
  STOP_DELIMITER,
  START,
  MAYBE_DELIMITER,
  DELIMITER,
  RESERVED_DELIMITER,
  OBJECT,
  ARRAY,
  COMMENT,
  ESCAPE,
  COMMENT_DELIMITER,
  ESCAPE_DELIMITER,
  ESCAPE_DELIMITER_NAME,
  INSTRUCTION_DELIMITER,
  INSTRUCTION_DELIMITER_NAME,
  INSTRUCTION_DELIMITER_ARGS,
  DATA_DELIMITER,
  DATA_DELIMITER_NAME,
  DATA_DELIMITER_ARGS,
  OBJECT_DELIMITER,
  ARRAY_DELIMITER,
  VOID_DELIMITER,
  PART_DELIMITER,
  DATA,
  GO,
  STOP,
}

type ASLANParserSettings = {
  prefix: string;
  defaultFieldName: string;
  eventListeners: ASLANEventListener[];
  strictStart: boolean;
  strictEnd: boolean;
  emittableEvents: {
    content: boolean;
    end: boolean;
    endData: boolean;
  };
};

enum ASLANDataInsertionType {
  DEFAULT,
  APPEND,
  KEEP_FIRST,
  KEEP_LAST,
}

function dataInsertionTypeToString(type: ASLANDataInsertionType) {
  switch (type) {
    case ASLANDataInsertionType.APPEND: return 'APPEND';
    case ASLANDataInsertionType.KEEP_FIRST: return 'KEEP_FIRST';
    case ASLANDataInsertionType.KEEP_LAST: return 'KEEP_LAST';
    default: return 'DEFAULT';
  }
}

type ASLANParserStateStack = {
  innerResult: ASLANObject | ASLANArray;
  dataInsertionTypes: { [key: string]: ASLANDataInsertionType };
  dataInsertionLocks: { [key: string]: boolean };
};

export class ASLANParser {
  private state: ASLANParserState = ASLANParserState.START;
  private result: ASLANObject = {
    _default: null,
  };
  private dataInsertionTypes: { [key: string]: ASLANDataInsertionType } = {
    _default: ASLANDataInsertionType.DEFAULT,
  };
  private dataInsertionLocks: { [key: string]: boolean } = {
    _default: false,
  };
  private stack: ASLANParserStateStack[] = [{
    innerResult: this.result,
    dataInsertionTypes: this.dataInsertionTypes,
    dataInsertionLocks: this.dataInsertionLocks,
  }];
  private currentKey: string = '_default';
  private currentDelimiter: ASLANDelimiterData | null = null;
  private currentValue: string = '';
  private delimiterBuffer: string = '';
  private delimiterOpenSubstring: string;
  private currentKeyVoid = false;

  constructor(public readonly parserSettings: ASLANParserSettings = {
    prefix: 'aslan',
    defaultFieldName: '_default',
    eventListeners: [],
    strictStart: true,
    strictEnd: true,
    emittableEvents: { content: true, end: true, endData: true },
  }) {
    this.delimiterOpenSubstring = '[' + parserSettings.prefix;
    this.currentKey = parserSettings.defaultFieldName;
  }

  parse(input: string): ASLANObject {
    for (let char of input) {
      this.handleNextChar(char);
    }
    this.close();
    return this.stack[0].innerResult as ASLANObject;
  }

  parseNext(input: string) {
    for (let char of input) {
      this.handleNextChar(char);
    }
  }

  getCurrentValue() {
    return this.currentValue;
  }

  private exitInvalidDelimiterIntoDATA(char: string) {
    this.currentValue += this.delimiterBuffer + char;
    this.delimiterBuffer = '';
    this.currentDelimiter = null;
    this.state = ASLANParserState.DATA;
  }

  private handleNextChar(char: string) {
    switch (this.state) {
      case ASLANParserState.GO_DELIMITER:
        this.handleGoDelimiter(char);
        break;
      case ASLANParserState.STOP_DELIMITER:
        this.handleStopDelimiter(char);
        break;
      case ASLANParserState.GO:
        this.handleGo(char);
        break;
      case ASLANParserState.STOP:
        this.handleStop(char);
        break;
      case ASLANParserState.START:
        this.handleStart(char);
        break;
      case ASLANParserState.MAYBE_DELIMITER:
        this.handleMaybeDelimiter(char);
        break;
      case ASLANParserState.DELIMITER:
        this.handleDelimiter(char);
        break;
      case ASLANParserState.RESERVED_DELIMITER:
        this.handleReservedDelimiter(char);
        break;
      case ASLANParserState.OBJECT:
        this.handleObject(char);
        break;
      case ASLANParserState.ARRAY:
        this.handleArray(char);
        break;
      case ASLANParserState.COMMENT:
        this.handleComment(char);
        break;
      case ASLANParserState.ESCAPE:
        this.handleEscape(char);
        break;
      case ASLANParserState.INSTRUCTION_DELIMITER:
        this.handleInstructionDelimiter(char);
        break;
      case ASLANParserState.INSTRUCTION_DELIMITER_NAME:
        this.handleInstructionDelimiterName(char);
        break;
      case ASLANParserState.INSTRUCTION_DELIMITER_ARGS:
        this.handleInstructionDelimiterArgs(char);
        break;
      case ASLANParserState.DATA_DELIMITER:
        this.handleDataDelimiter(char);
        break;
      case ASLANParserState.DATA_DELIMITER_NAME:
        this.handleDataDelimiterName(char);
        break;
      case ASLANParserState.DATA_DELIMITER_ARGS:
        this.handleDataDelimiterArgs(char);
        break;
      case ASLANParserState.OBJECT_DELIMITER:
        this.handleObjectDelimiter(char);
        break;
      case ASLANParserState.ARRAY_DELIMITER:
        this.handleArrayDelimiter(char);
        break;
      case ASLANParserState.VOID_DELIMITER:
        this.handleVoidDelimiter(char);
        break;
      case ASLANParserState.COMMENT_DELIMITER:
        this.handleCommentDelimiter(char);
        break;
      case ASLANParserState.ESCAPE_DELIMITER:
        this.handleEscapeDelimiter(char);
        break;
      case ASLANParserState.ESCAPE_DELIMITER_NAME:
        this.handleEscapeDelimiterName(char);
        break;
      case ASLANParserState.PART_DELIMITER:
        this.handlePartDelimiter(char);
        break;
      case ASLANParserState.DATA:
        this.handleData(char);
        break;
    }
  }

  private handleGoDelimiter(char: string) {
    if (char === ']') {
      //Spec: Go delimiters have no <CONTENT> or args
      //VALID GO DELIMITER
      this.state = ASLANParserState.GO;
      this.delimiterBuffer = '';
      this.currentValue = '';
      return;
    }
    //Spec: Go delimiters have no <CONTENT> or args
    //INVALID GO DELIMITER
    this.exitInvalidDelimiterIntoDATA(char);
  }

  private handleStopDelimiter(char: string) {
    if (char === ']') {
      //Spec: Stop delimiters have no <CONTENT> or args
      //VALID STOP DELIMITER
      this.state = ASLANParserState.STOP;
      this.delimiterBuffer = '';
      this.currentValue = '';
      return;
    }
    //Spec: Stop delimiters have no <CONTENT> or args
    //INVALID STOP DELIMITER
    this.exitInvalidDelimiterIntoDATA(char);
  }

  private handleStart(char: string) {
    if (char === '[') {
      this.state = ASLANParserState.MAYBE_DELIMITER;
      this.delimiterBuffer += char;
    } else {
      this.state = ASLANParserState.DATA;
      this.currentValue += char;
    }
  }

  private handleMaybeDelimiter(char: string) {
    if (this.delimiterBuffer.length > this.delimiterOpenSubstring.length) {
      this.state = ASLANParserState.DATA;
      this.currentValue += char;
      return;
    }
    if (char === this.delimiterOpenSubstring[this.delimiterBuffer.length]) {
      this.delimiterBuffer += char;
      if (this.delimiterBuffer === this.delimiterOpenSubstring) {
        this.state = ASLANParserState.DELIMITER;
      }
      return;
    }
    this.state = ASLANParserState.DATA;
    this.currentValue += char;
  }

  private handleDelimiter(char: string) {
    this.currentDelimiter = {
      prefix: this.parserSettings.prefix,
      suffix: null,
      content: null,
      args: [],
    };
    switch (char) {
      case 'd':
        this.state = ASLANParserState.DATA_DELIMITER;
        this.currentDelimiter.suffix = ASLANDelimiterType.DATA;
        this.delimiterBuffer += char;
        break;
      case 'o':
        this.state = ASLANParserState.OBJECT_DELIMITER;
        this.currentDelimiter.suffix = ASLANDelimiterType.OBJECT;
        this.delimiterBuffer += char;
        break;
      case 'i':
        this.state = ASLANParserState.INSTRUCTION_DELIMITER;
        this.currentDelimiter.suffix = ASLANDelimiterType.INSTRUCTION;
        this.delimiterBuffer += char;
        break;
      case 'a':
        this.state = ASLANParserState.ARRAY_DELIMITER;
        this.currentDelimiter.suffix = ASLANDelimiterType.ARRAY;
        this.delimiterBuffer += char;
        break;
      case 'c':
        this.state = ASLANParserState.COMMENT_DELIMITER;
        this.currentDelimiter.suffix = ASLANDelimiterType.COMMENT;
        this.delimiterBuffer += char;
        break;
      case 'e':
        this.state = ASLANParserState.ESCAPE_DELIMITER;
        this.currentDelimiter.suffix = ASLANDelimiterType.ESCAPE;
        this.delimiterBuffer += char;
        break;
      case 'p':
        this.state = ASLANParserState.PART_DELIMITER;
        this.currentDelimiter.suffix = ASLANDelimiterType.PART;
        this.delimiterBuffer += char;
        break;
      case 'v':
        this.state = ASLANParserState.VOID_DELIMITER;
        this.currentDelimiter.suffix = ASLANDelimiterType.VOID;
        this.delimiterBuffer += char;
        break;
      case 'g':
        this.state = ASLANParserState.GO_DELIMITER;
        this.currentDelimiter.suffix = ASLANDelimiterType.GO;
        this.delimiterBuffer += char;
        break;
      case 's':
        this.state = ASLANParserState.STOP_DELIMITER;
        this.currentDelimiter.suffix = ASLANDelimiterType.STOP;
        this.delimiterBuffer += char;
        break;
      default:
        if (/^[a-zA-Z0-9]$/.test(char)) {
          this.state = ASLANParserState.RESERVED_DELIMITER;
          this.delimiterBuffer += char;
          return;
        }
        this.state = ASLANParserState.DATA;
        this.currentValue += char;
        break;
    }
  }

  handleReservedDelimiter(char: string) {
    if (char !== ']') {
      //Spec: Reserved delimiters contain no <CONTENT> or args
      //INVALID RESERVED DELIMITER
      return this.exitInvalidDelimiterIntoDATA(char);
    }
    this.delimiterBuffer = '';
    this.state = ASLANParserState.DATA;
    this.currentValue = '';
  }

  handleObjectDelimiter(char: string) {
    if (char === ']') {
      //Spec: Object delimiters have no <CONTENT> or args
      //VALID OBJECT DELIMITER
      this.state = ASLANParserState.OBJECT;
      this.delimiterBuffer = '';
      if(this.getLatestResult()[this.currentKey]) {
        if(this.stack.length > 1) {
          this.stack.pop();
        }
        return;
      }
      this.currentValue = '';
      this.getLatestResult()[this.currentKey] = {};
      this.stack.push({
        innerResult: this.getLatestResult()[this.currentKey] as ASLANObject,
        dataInsertionTypes: {},
        dataInsertionLocks: {},
      });
      return;
    }
    //Spec: Object delimiters have no <CONTENT> or args
    //INVALID OBJECT DELIMITER
    this.exitInvalidDelimiterIntoDATA(char);
  }

  handleInstructionDelimiter(char: string) {
    if (char === ']') {
      //Spec: Instruction delimiters must contain <CONTENT>
      //INVALID INSTRUCTION DELIMITER
      return this.exitInvalidDelimiterIntoDATA(char);
    }
    if (char === '_') {
      //Spec: Instruction delimiters must contain <CONTENT>
      this.state = ASLANParserState.INSTRUCTION_DELIMITER_NAME;
      this.delimiterBuffer += char;
      this.currentDelimiter!.content = '';
      this.currentValue = '';
      return;
    }
    //Spec: Instruction delimiters must contain <CONTENT>
    //INVALID INSTRUCTION DELIMITER
    this.exitInvalidDelimiterIntoDATA(char);
  }

  handleInstructionDelimiterName(char: string) {
    if (this.currentDelimiter!.content !== '' && char === ':') {
      if (this.currentDelimiter!.content?.endsWith('_')) {
        //Spec: Delimiter <CONTENT> may not end with an underscore.
        //INVALID INSTRUCTION DELIMITER
        return this.exitInvalidDelimiterIntoDATA(char);
      }
      //Spec: Instructions may have arguments.
      this.state = ASLANParserState.INSTRUCTION_DELIMITER_ARGS;
      this.currentDelimiter!.args = [''];
      this.currentValue = '';
      this.delimiterBuffer += char;
      return;
    }
    if (char === '_' && this.currentDelimiter!.content === '') {
      //Spec: Delimiter <CONTENT> may not start with an underscore.
      //INVALID INSTRUCTION DELIMITER
      return this.exitInvalidDelimiterIntoDATA(char);
    }
    if (char === ']') {
      if (this.currentDelimiter!.content?.endsWith('_')) {
        //Spec: Delimiter <CONTENT> may not end with an underscore.
        //INVALID INSTRUCTION DELIMITER
        return this.exitInvalidDelimiterIntoDATA(char);
      }
      //Spec: Instruction delimiter of the form [<PREFIX>i_<CONTENT>]
      //VALID INSTRUCTION DELIMITER
      this.state = ASLANParserState.DATA;
      this.delimiterBuffer = '';
      this.currentValue = '';
      return;
    }
    if (!/^[a-zA-Z0-9_]$/.test(char)) {
      //Spec: Delimiter <CONTENT> may only contain alphanumeric characters and underscores.
      //INVALID INSTRUCTION DELIMITER
      return this.exitInvalidDelimiterIntoDATA(char);
    }
    this.currentDelimiter!.content += char;
    this.delimiterBuffer += char;
  }

  handleInstructionDelimiterArgs(char: string) {
    if (char === ']') {
      //Spec: Instruction delimiter of the form [<PREFIX>i_<CONTENT>:<ARG0>:<ARG1>:<ARG2>:...]
      //VALID INSTRUCTION DELIMITER
      this.state = ASLANParserState.DATA;
      this.delimiterBuffer = '';
      this.currentValue = '';
      return;
    }
    if (char === ':') {
      //Start a new arg
      this.delimiterBuffer += char;
      this.currentDelimiter!.args.push('');
      return;
    }
    //Add to the current arg
    this.currentDelimiter!.args[this.currentDelimiter!.args.length - 1] += char;
    this.delimiterBuffer += char;
  }

  handleDataDelimiter(char: string) {
    if (char === ']') {
      //Spec: Data delimiters must contain <CONTENT>
      //INVALID DATA DELIMITER
      return this.exitInvalidDelimiterIntoDATA(char);
    }
    if (char === '_') {
      this.state = ASLANParserState.DATA_DELIMITER_NAME;
      this.delimiterBuffer += char;
      this.currentDelimiter!.content = '';
      this.currentValue = '';
      return;
    }
    //Spec: Data delimiters must be valid of the form [<PREFIX>d_<CONTENT>] or [<PREFIX>d_<CONTENT>:<ARG0>:<ARG1>:<ARG2>:...]
    //INVALID DATA DELIMITER
    this.exitInvalidDelimiterIntoDATA(char);
  }

  handleDataDelimiterName(char: string) {
    if (this.currentDelimiter!.content !== '' && char === ':') {
      if (this.currentDelimiter!.content?.endsWith('_')) {
        //Spec: Delimiter <CONTENT> may not end with an underscore.
        //INVALID DATA DELIMITER
        return this.exitInvalidDelimiterIntoDATA(char);
      }
      //Spec: Data may have arguments.
      this.state = ASLANParserState.DATA_DELIMITER_ARGS;
      this.currentDelimiter!.args = [''];
      this.currentValue = '';
      this.delimiterBuffer += char;
      this.nextKey();
      return;
    }
    if (char === '_' && this.currentDelimiter!.content === '') {
      //Spec: Delimiter <CONTENT> may not start with an underscore.
      //INVALID DATA DELIMITER
      return this.exitInvalidDelimiterIntoDATA(char);
    }
    if (char === ']') {
      if (this.currentDelimiter!.content?.endsWith('_')) {
        //Spec: Delimiter <CONTENT> may not end with an underscore.
        //INVALID DATA DELIMITER
        return this.exitInvalidDelimiterIntoDATA(char);
      }
      //Spec: Data delimiter of the form [<PREFIX>d_<CONTENT>]
      //VALID DATA DELIMITER
      this.state = ASLANParserState.DATA;
      this.nextKey();
      this.delimiterBuffer = '';
      this.currentValue = '';
      this.setDataInsertionType(ASLANDataInsertionType.DEFAULT);
      return;
    }
    if (!/^[a-zA-Z0-9_]$/.test(char)) {
      //Spec: Delimiter <CONTENT> may only contain alphanumeric characters and underscores.
      //INVALID DATA DELIMITER
      return this.exitInvalidDelimiterIntoDATA(char);
    }
    this.currentDelimiter!.content += char;
    this.delimiterBuffer += char;
  }

  handleDataDelimiterArgs(char: string) {
    if (char === ']') {
      //Spec: Data delimiter of the form [<PREFIX>d_<CONTENT>:<ARG0>:<ARG1>:<ARG2>:...]
      //VALID DATA DELIMITER
      this.state = ASLANParserState.DATA;
      this.delimiterBuffer = '';
      this.currentValue = '';
      const arg = this.currentDelimiter?.args[0] as ASLANDuplicateKeyBehavior;
      switch (arg) {
        case 'a':
          this.setDataInsertionType(ASLANDataInsertionType.APPEND);
          break;
        case 'f':
          this.setDataInsertionType(ASLANDataInsertionType.KEEP_FIRST);
          break;
        case 'l':
          this.setDataInsertionType(ASLANDataInsertionType.KEEP_LAST);
          break;
        default:
          this.setDataInsertionType(ASLANDataInsertionType.DEFAULT);
          break;
      }
      return;
    }
    if (char === ':') {
      //Start a new arg
      this.delimiterBuffer += char;
      this.currentDelimiter!.args.push('');
      return;
    }
    //Add to the current arg
    this.currentDelimiter!.args[this.currentDelimiter!.args.length - 1] += char;
    this.delimiterBuffer += char;
  }

  handleArrayDelimiter(char: string) {
    if (char === ']') {
      //Spec: Array delimiters have no <CONTENT> or args
      //VALID ARRAY DELIMITER
      this.state = ASLANParserState.ARRAY;
      this.delimiterBuffer = '';
      this.currentValue = '';
      return;
    }
    //Spec: Array delimiters have no <CONTENT> or args
    //INVALID ARRAY DELIMITER
    this.exitInvalidDelimiterIntoDATA(char);
  }

  handleVoidDelimiter(char: string) {
    if (char === ']') {
      //Spec: Void delimiters have no <CONTENT> or args
      //VALID VOID DELIMITER
      this.state = ASLANParserState.DATA;
      this.delimiterBuffer = '';
      this.currentValue = '';
      this.currentKeyVoid = true;
      return;
    }
    //Spec: Void delimiters have no <CONTENT> or args
    //INVALID VOID DELIMITER
    this.exitInvalidDelimiterIntoDATA(char);
  }

  handleCommentDelimiter(char: string) {
    if (char === ']') {
      //Spec: Comment delimiters have no <CONTENT> or args
      //VALID COMMENT DELIMITER
      this.state = ASLANParserState.COMMENT;
      this.delimiterBuffer = '';
      this.currentValue = '';
      return;
    }
    //Spec: Comment delimiters have no <CONTENT> or args
    //INVALID COMMENT DELIMITER
    this.exitInvalidDelimiterIntoDATA(char);
  }

  handleEscapeDelimiter(char: string) {
    if (char === ']') {
      //Spec: Escape delimiters must contain <CONTENT>
      //INVALID ESCAPE DELIMITER
      return this.exitInvalidDelimiterIntoDATA(char);
    }
    if (char === '_') {
      this.state = ASLANParserState.ESCAPE_DELIMITER_NAME;
      this.delimiterBuffer += char;
      this.currentDelimiter!.content = '';
      this.currentValue = '';
      return;
    }
    //Spec: Escape delimiters must be valid of the form [<PREFIX>e_<CONTENT>]
    //INVALID ESCAPE DELIMITER
    this.exitInvalidDelimiterIntoDATA(char);
  }

  handleEscapeDelimiterName(char: string) {
    if (char === '_' && this.currentDelimiter!.content === '') {
      //Spec: Delimiter <CONTENT> may not start with an underscore.
      //INVALID ESCAPE DELIMITER
      return this.exitInvalidDelimiterIntoDATA(char);
    }
    if (char === ']') {
      if (this.currentDelimiter!.content?.endsWith('_')) {
        //Spec: Delimiter <CONTENT> may not end with an underscore.
        //INVALID ESCAPE DELIMITER
        return this.exitInvalidDelimiterIntoDATA(char);
      }
      //Spec: Escape delimiter of the form [<PREFIX>e_<CONTENT>]
      //VALID ESCAPE DELIMITER
      this.state = ASLANParserState.ESCAPE;
      this.delimiterBuffer = '';
      this.currentValue = '';
      return;
    }
    if (!/^[a-zA-Z0-9_]$/.test(char)) {
      //Spec: Delimiter <CONTENT> may only contain alphanumeric characters and underscores.
      //INVALID ESCAPE DELIMITER
      return this.exitInvalidDelimiterIntoDATA(char);
    }
    this.currentDelimiter!.content += char;
    this.delimiterBuffer += char;
  }

  handlePartDelimiter(char: string) {
    if (char === ']') {
      //Spec: Part delimiters have no <CONTENT> or args
      //VALID PART DELIMITER
      this.state = ASLANParserState.DATA;
      this.delimiterBuffer = '';
      this.currentValue = '';
      return;
    }
    //Spec: Part delimiters have no <CONTENT> or args
    //INVALID PART DELIMITER
    this.exitInvalidDelimiterIntoDATA(char);
  }

  handleGo(char: string) {
    if (char === '[') {
      this.state = ASLANParserState.MAYBE_DELIMITER;
      this.delimiterBuffer += char;
      return;
    }
    this.appendToCurrentValue(char);
  }

  handleStop(char: string) {
    if (char === '[') {
      this.state = ASLANParserState.MAYBE_DELIMITER;
      this.delimiterBuffer += char;
      return;
    }
    this.appendToCurrentValue(char);
  }

  handleObject(char: string) {
    if (char === '[') {
      this.state = ASLANParserState.MAYBE_DELIMITER;
      this.delimiterBuffer += char;
      return;
    }
    this.appendToCurrentValue(char);
  }

  handleArray(char: string) {
    if (char === '[') {
      this.state = ASLANParserState.MAYBE_DELIMITER;
      this.delimiterBuffer += char;
      return;
    }
    this.appendToCurrentValue(char);
  }

  handleComment(char: string) {
    if (char === '[') {
      this.state = ASLANParserState.MAYBE_DELIMITER;
      this.delimiterBuffer += char;
    }
  }

  handleEscape(char: string) {
    if (char === '[') {
      this.state = ASLANParserState.MAYBE_DELIMITER;
      this.delimiterBuffer += char;
    }
    this.appendToCurrentValue(char);
  }

  handleData(char: string) {
    if (char === '[') {
      this.state = ASLANParserState.MAYBE_DELIMITER;
      this.delimiterBuffer += char;
      return;
    }
    this.appendToCurrentValue(char);
    this.storeCurrentValue();
  }

  private setCurrentValue(value: string) {
    this.currentValue = value;
  }

  private appendToCurrentValue(value: string) {
    this.currentValue += value;
  }

  private storeCurrentValue() {
    if (this.currentKeyVoid) {
      this.currentValue = '';
      this.getLatestResult()[this.currentKey] = null;
      return;
    }
    if (this.currentValue) {
      if (!this.getLatestResult()[this.currentKey]) {
        this.getLatestResult()[this.currentKey] = '';
      }
      if (!this.dataInsertionLocks[this.currentKey]) {
        this.getLatestResult()[this.currentKey] += this.currentValue;
      }
      this.currentValue = '';
    }
  }

  private setDataInsertionType(type: ASLANDataInsertionType) {
    //Spec: Data insertion type can only be set once for a given key in an object/array.
    //NOTE: We keep the behavior as defined on the first occurrence of the key to avoid LLM instability causing difficult to predict behavior.
    if (this.dataInsertionTypes[this.currentKey] !== undefined) {
      // If we're trying to set the type again then we've hit a duplicate key so check if it's a KEEP_LAST and clear the value if so
      // Otherwise if it's KEEP_FIRST lock future appearances of the key.
      switch (this.dataInsertionTypes[this.currentKey]) {
        case ASLANDataInsertionType.KEEP_LAST:
          this.getLatestResult()[this.currentKey] = '';
          break;
        case ASLANDataInsertionType.KEEP_FIRST:
          this.dataInsertionLocks[this.currentKey] = true;
          break;
        default:
          break;
      }
      return;
    }
    this.dataInsertionTypes[this.currentKey] = type;
  }

  private nextKey() {
    if (this.currentDelimiter?.content) {
      this.currentKey = this.currentDelimiter.content;
    }
    this.currentKeyVoid = false;
  }

  close() {
    this.storeCurrentValue();
  }

  getResult() {
    return this.stack[0].innerResult;
  }

  private getLatestResult() {
    return this.stack[this.stack.length - 1].innerResult as ASLANObject;
  }
}

const parser = new ASLANParser();
const result = parser.parse(
  '[aslani_test:hello:2:9:]hello[aslani_test2]world[asland]test[asland_me]Hi'
);
