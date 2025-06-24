import { Lexer, Token, TokenType } from './lexer.js';

export interface FormatterOptions {
  indentSize: number;
  maxLineLength: number;
  removeTrailingWhitespace: boolean;
  normalizeSpacing: boolean;
  alignArrows: boolean;
}

export const defaultFormatterOptions: FormatterOptions = {
  indentSize: 2,
  maxLineLength: 80,
  removeTrailingWhitespace: true,
  normalizeSpacing: true,
  alignArrows: false,
};

export class SeseragiFormatter {
  private options: FormatterOptions;
  private tokens: Token[] = [];
  private position = 0;
  private indentLevel = 0;
  private output: string[] = [];

  constructor(options: FormatterOptions = defaultFormatterOptions) {
    this.options = { ...defaultFormatterOptions, ...options };
  }

  format(code: string): string {
    const lexer = new Lexer(code);
    this.tokens = lexer.tokenize();
    this.position = 0;
    this.indentLevel = 0;
    this.output = [];

    while (!this.isAtEnd()) {
      this.formatStatement();
    }

    let result = this.output.join('');
    
    if (this.options.removeTrailingWhitespace) {
      result = this.removeTrailingWhitespace(result);
    }

    return result;
  }

  private formatStatement(): void {
    this.skipWhitespaceAndComments();

    if (this.isAtEnd()) return;

    const token = this.current();

    switch (token.type) {
      case TokenType.FN:
        this.formatFunction();
        break;
      case TokenType.LET:
        this.formatLetBinding();
        break;
      case TokenType.TYPE:
        this.formatTypeDefinition();
        break;
      case TokenType.IMPL:
        this.formatImplBlock();
        break;
      case TokenType.MONOID:
        this.formatMonoidBlock();
        break;
      case TokenType.COMMENT:
        this.formatComment();
        break;
      case TokenType.NEWLINE:
        this.addNewline();
        this.advance();
        break;
      default:
        this.advance();
    }
  }

  private formatFunction(): void {
    this.addIndent();
    this.addToken(); // 'fn'
    this.addSpace();
    
    // Function name
    this.addToken();
    
    // Parameters
    while (!this.isAtEnd() && !this.check(TokenType.EQUALS)) {
      if (this.check(TokenType.ARROW)) {
        this.addSpace();
        this.addToken();
        this.addSpace();
      } else if (this.check(TokenType.COLON)) {
        this.addSpace();
        this.addToken();
      } else {
        this.addToken();
      }
    }
    
    if (this.check(TokenType.EQUALS)) {
      this.addSpace();
      this.addToken(); // '='
      this.addSpace();
      
      // Function body
      if (this.checkNext(TokenType.NEWLINE) || this.isMultilineExpression()) {
        this.addNewline();
        this.indentLevel++;
        this.formatExpression();
        this.indentLevel--;
      } else {
        this.formatExpression();
      }
    }
    
    this.addNewline();
  }

  private formatLetBinding(): void {
    this.addIndent();
    this.addToken(); // 'let'
    this.addSpace();
    
    // Variable name
    this.addToken();
    
    // Type annotation
    if (this.check(TokenType.COLON)) {
      this.addSpace();
      this.addToken();
    }
    
    // Assignment
    if (this.check(TokenType.EQUALS)) {
      this.addSpace();
      this.addToken();
      this.addSpace();
      this.formatExpression();
    }
    
    this.addNewline();
  }

  private formatTypeDefinition(): void {
    this.addIndent();
    this.addToken(); // 'type'
    this.addSpace();
    this.addToken(); // type name
    this.addSpace();
    this.addToken(); // '='
    
    if (this.checkNext(TokenType.NEWLINE) || this.check(TokenType.PIPE)) {
      this.addNewline();
      this.indentLevel++;
      
      while (this.check(TokenType.PIPE)) {
        this.addIndent();
        this.addToken(); // '|'
        this.addSpace();
        this.formatTypeConstructor();
        this.addNewline();
      }
      
      this.indentLevel--;
    } else {
      this.addSpace();
      this.formatTypeConstructor();
      this.addNewline();
    }
  }

  private formatImplBlock(): void {
    this.addIndent();
    this.addToken(); // 'impl'
    this.addSpace();
    this.addToken(); // type name
    this.addSpace();
    this.addToken(); // '{'
    this.addNewline();
    
    this.indentLevel++;
    while (!this.check(TokenType.RBRACE) && !this.isAtEnd()) {
      this.formatStatement();
    }
    this.indentLevel--;
    
    this.addIndent();
    this.addToken(); // '}'
    this.addNewline();
  }

  private formatMonoidBlock(): void {
    this.addIndent();
    this.addToken(); // 'monoid'
    this.addSpace();
    this.addToken(); // monoid name
    this.addSpace();
    this.addToken(); // '{'
    this.addNewline();
    
    this.indentLevel++;
    while (!this.check(TokenType.RBRACE) && !this.isAtEnd()) {
      if (this.check(TokenType.IDENTIFIER)) {
        this.addIndent();
        this.addToken(); // property name
        this.addSpace();
        this.addToken(); // '='
        this.addSpace();
        this.formatExpression();
        this.addNewline();
      } else {
        this.advance();
      }
    }
    this.indentLevel--;
    
    this.addIndent();
    this.addToken(); // '}'
    this.addNewline();
  }

  private formatExpression(): void {
    while (!this.isAtEnd() && !this.isStatementEnd()) {
      if (this.check(TokenType.PIPE)) {
        if (this.shouldBreakPipeline()) {
          this.addNewline();
          this.addIndent();
          this.addToken();
          this.addSpace();
        } else {
          this.addSpace();
          this.addToken();
          this.addSpace();
        }
      } else if (this.check(TokenType.MATCH)) {
        this.formatMatchExpression();
      } else if (this.check(TokenType.IS)) {
        this.addSpace();
        this.addToken();
        this.addSpace();
      } else if (this.check(TokenType.THEN)) {
        this.addSpace();
        this.addToken();
        this.addSpace();
      } else if (this.check(TokenType.ELSE)) {
        this.addSpace();
        this.addToken();
        this.addSpace();
      } else if (this.check(TokenType.LPAREN)) {
        this.addToken();
        this.formatExpression();
        if (this.check(TokenType.RPAREN)) {
          this.addToken();
        }
      } else {
        this.addToken();
      }
    }
  }

  private formatMatchExpression(): void {
    this.addToken(); // 'match'
    this.addSpace();
    
    // Match expression
    while (!this.check(TokenType.NEWLINE) && !this.check(TokenType.PIPE) && !this.isAtEnd()) {
      this.addToken();
    }
    
    this.addNewline();
    this.indentLevel++;
    
    // Match cases
    while (this.check(TokenType.PIPE) && !this.isAtEnd()) {
      this.addIndent();
      this.addToken(); // '|'
      this.addSpace();
      
      // Pattern
      while (!this.check(TokenType.ARROW) && !this.isAtEnd()) {
        this.addToken();
      }
      
      if (this.check(TokenType.ARROW)) {
        this.addSpace();
        this.addToken();
        this.addSpace();
        
        // Expression
        this.formatExpression();
        this.addNewline();
      }
    }
    
    this.indentLevel--;
  }

  private formatTypeConstructor(): void {
    while (!this.check(TokenType.NEWLINE) && !this.check(TokenType.PIPE) && !this.isAtEnd()) {
      this.addToken();
    }
  }

  private formatComment(): void {
    this.addIndent();
    this.addToken();
    this.addNewline();
  }

  private isMultilineExpression(): boolean {
    const saved = this.position;
    let tokenCount = 0;
    
    while (!this.isStatementEnd() && !this.isAtEnd() && tokenCount < 10) {
      if (this.check(TokenType.MATCH) || this.check(TokenType.NEWLINE)) {
        this.position = saved;
        return true;
      }
      this.advance();
      tokenCount++;
    }
    
    this.position = saved;
    return false;
  }

  private shouldBreakPipeline(): boolean {
    let lineLength = this.getCurrentLineLength();
    const saved = this.position;
    
    while (!this.check(TokenType.NEWLINE) && !this.isStatementEnd() && !this.isAtEnd()) {
      lineLength += this.current().value.length;
      this.advance();
    }
    
    this.position = saved;
    return lineLength > this.options.maxLineLength;
  }

  private isStatementEnd(): boolean {
    return this.check(TokenType.NEWLINE) || 
           this.check(TokenType.EOF) ||
           this.check(TokenType.RBRACE);
  }

  private getCurrentLineLength(): number {
    const currentIndent = this.indentLevel * this.options.indentSize;
    const lastNewlineIndex = this.output.lastIndexOf('\n');
    
    if (lastNewlineIndex === -1) {
      return this.output.join('').length + currentIndent;
    }
    
    return this.output.slice(lastNewlineIndex + 1).join('').length;
  }

  private addIndent(): void {
    this.output.push(' '.repeat(this.indentLevel * this.options.indentSize));
  }

  private addSpace(): void {
    if (this.options.normalizeSpacing) {
      this.output.push(' ');
    }
  }

  private addNewline(): void {
    this.output.push('\n');
  }

  private addToken(): void {
    if (!this.isAtEnd()) {
      this.output.push(this.current().value);
      this.advance();
    }
  }

  private removeTrailingWhitespace(text: string): string {
    return text.split('\n')
      .map(line => line.trimEnd())
      .join('\n');
  }

  private skipWhitespaceAndComments(): void {
    while (!this.isAtEnd() && 
           (this.check(TokenType.WHITESPACE) || 
            this.check(TokenType.COMMENT))) {
      if (this.check(TokenType.COMMENT)) {
        break; // Let formatComment handle it
      }
      this.advance();
    }
  }

  private current(): Token {
    return this.tokens[this.position];
  }

  private check(type: TokenType): boolean {
    return !this.isAtEnd() && this.current().type === type;
  }

  private checkNext(type: TokenType): boolean {
    return this.position + 1 < this.tokens.length && 
           this.tokens[this.position + 1].type === type;
  }

  private advance(): void {
    if (!this.isAtEnd()) {
      this.position++;
    }
  }

  private isAtEnd(): boolean {
    return this.position >= this.tokens.length || 
           this.current().type === TokenType.EOF;
  }
}