# -*- coding: utf-8 -*-
#
# Pygments lexer for Cretonne.
from __future__ import absolute_import

from pygments.lexer import RegexLexer, bygroups, words
from pygments.token import Comment, String, Keyword, Whitespace, Number, Name
from pygments.token import Operator, Punctuation, Text


def keywords(*args):
    return words(args, prefix=r'\b', suffix=r'\b')


class CretonneLexer(RegexLexer):
    name = 'Cretonne'
    aliases = ['cton']
    filenames = ['*.cton']

    tokens = {
        'root': [
            # Test header lines.
            (r'^(test|isa|set)(?:( +)([-\w]+)' +
             r'(?:(=)(?:(\d+)|(yes|no|true|false|on|off)|(\w+)))?)*' +
             r'( *)$',
                bygroups(Keyword.Namespace, Whitespace, Name.Attribute,
                         Operator, Number.Integer, Keyword.Constant,
                         Name.Constant, Whitespace)),
            # Comments with filecheck or other test directive.
            (r'(; *)([a-z]+:)(.*?)$',
                bygroups(Comment.Single, Comment.Special, Comment.Single)),
            # Plain comments.
            (r';.*?$', Comment.Single),
            # Strings are in double quotes, support \xx escapes only.
            (r'"([^"\\]+|\\[0-9a-fA-F]{2})*"', String),
            # A naked function name following 'function' is also a string.
            (r'\b(function)([ \t]+)(\w+)\b',
                bygroups(Keyword, Whitespace, String.Symbol)),
            # Numbers.
            (r'[-+]?0[xX][0-9a-fA-F]+', Number.Hex),
            (r'[-+]?0[xX][0-9a-fA-F]*\.[0-9a-fA-F]*([pP]\d+)?', Number.Hex),
            (r'[-+]?(\d+\.\d+([eE]\d+)?|[sq]NaN|Inf)', Number.Float),
            (r'[-+]?\d+', Number.Integer),
            # Known attributes.
            (keywords('align', 'aligntrap', 'uext', 'sext', 'inreg'),
                Name.Attribute),
            # Well known value types.
            (r'\b(b\d+|i\d+|f32|f64)(x\d+)?\b', Keyword.Type),
            # v<nn> = value
            # vx<nn> = value
            # ss<nn> = stack slot
            # jt<nn> = jump table
            (r'(vx?|ss|jt)\d+', Name.Variable),
            # ebb<nn> = extended basic block
            (r'(ebb)\d+', Name.Label),
            # Match instruction names in context.
            (r'(=)( *)([a-z]\w*)',
                bygroups(Operator, Whitespace, Name.Function)),
            (r'^( *)([a-z]\w*\b)(?! *[,=])',
                bygroups(Whitespace, Name.Function)),
            # Other names: results and arguments
            (r'[a-z]\w*', Name),
            (r'->|=|:', Operator),
            (r'[{}(),.]', Punctuation),
            (r'[ \t]+', Text),
        ],
    }


def setup(app):
    """Setup Sphinx extension."""
    app.add_lexer('cton', CretonneLexer())

    return {'version': '0.1'}
