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

class ASLANParser {
  private state: ASLANParserState = ASLANParserState.START;
  private prefix: string;
  private result: ASLANObject = {};
  private stack: (ASLANObject | ASLANArray)[] = [this.result];
  private currentKey: string = '_default';
  private currentDelimiter: ASLANDelimiterData | null = null;
  private currentValue: string = '';
  private delimiterBuffer: string = '';
  private delimiterOpenSubstring: string;

  constructor(prefix: string = 'aslan') {
    this.prefix = prefix;
    this.delimiterOpenSubstring = '[' + this.prefix;
  }

  parse(input: string): ASLANObject {
    for (let char of input) {
      this.handleNextChar(char);
    }
    return this.result;
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

  private handleGoDelimiter(char: string) {}

  private handleStopDelimiter(char: string) {}

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
      prefix: this.prefix,
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
      console.log('valid object delimiter', this.delimiterBuffer);
      this.state = ASLANParserState.OBJECT;
      this.delimiterBuffer = '';
      this.currentValue = '';
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
      console.log('valid instruction delimiter', this.delimiterBuffer);
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
      console.log(
        'valid instruction delimiter',
        this.delimiterBuffer,
        this.currentDelimiter
      );
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
      console.log('valid data delimiter', this.delimiterBuffer);
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
      console.log('valid data delimiter', this.delimiterBuffer);
      this.state = ASLANParserState.DATA;
      this.delimiterBuffer = '';
      this.currentValue = '';
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
      console.log(
        'valid data delimiter',
        this.delimiterBuffer,
        this.currentDelimiter
      );
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

  handleArrayDelimiter(char: string) {
    if (char === ']') {
      //Spec: Array delimiters have no <CONTENT> or args
      //VALID ARRAY DELIMITER
      console.log('valid array delimiter', this.delimiterBuffer);
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
      console.log('valid void delimiter', this.delimiterBuffer);
      this.state = ASLANParserState.DATA;
      this.delimiterBuffer = '';
      this.currentValue = '';
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
      console.log('valid comment delimiter', this.delimiterBuffer);
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
      console.log(
        'valid escape delimiter',
        this.delimiterBuffer,
        this.currentDelimiter
      );
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
      console.log('valid part delimiter', this.delimiterBuffer);
      this.state = ASLANParserState.DATA;
      this.delimiterBuffer = '';
      this.currentValue = '';
      return;
    }
    //Spec: Part delimiters have no <CONTENT> or args
    //INVALID PART DELIMITER
    this.exitInvalidDelimiterIntoDATA(char);
  }

  handleObject(char: string) {
    throw new Error('Method not implemented.');
  }

  handleArray(char: string) {
    throw new Error('Method not implemented.');
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
    this.currentValue += char;
  }

  handleData(char: string) {
    if (char === '[') {
      this.state = ASLANParserState.MAYBE_DELIMITER;
      this.delimiterBuffer += char;
      return;
    }
    this.currentValue += char;
  }
}

const parser = new ASLANParser();
const result = parser.parse(
  '[aslani_test:hello:2:9:]hello[aslani_test2]world[asland]test'
);
console.log(result);
console.log(parser.getCurrentValue());
