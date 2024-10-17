from dataclasses import dataclass
import random
from typing import Optional

@dataclass
class Word:
    text: str

    def __str__(self):
        return self.text

    def __eq__(self, value: object):
        return self.text == value.text

    def __hash__(self):
        return hash("WORD:" + self.text)

@dataclass
class End:

    def __str__(self):
        return "~END"
    
    def __hash__(self):
        return hash("END")

Token = Word | End

def token_is_word(t: Token) -> bool:
    match t:
        case Word(_): 
            return True
        case _:
            return False

def token_is_end(t: Token) -> bool:
    match t:
        case End():
            return True
        case _:
            return False

@dataclass
class StateTransitions:
    to_tokens: dict[Token, int]

    def register_transition(self, to_token: Token):
        if self.to_tokens.get(to_token) is None:
            self.to_tokens[to_token] = 0
        self.to_tokens[to_token] += 1

    def pick_token(self, allow_end: bool = False) -> Optional[Token]:
        entries = [e for e in self.to_tokens.items() if allow_end or not token_is_end(e[0])]
        if len(entries) == 0:
            return None
        else:
            return random.choices([k for (k, _) in entries], weights=[v for (_, v) in entries])[0]

@dataclass
class MarkovChain:
    edges: dict[Token, StateTransitions]

    def _update(self, from_token: Token, to_token: Token):
        if self.edges.get(from_token) is None:
            new = StateTransitions({})
            new.register_transition(to_token)
            self.edges[from_token] = new
        else:
            self.edges[from_token].register_transition(to_token)

    def _learn_from_tokens(self, tokens: list[Token]):
        for (i, token) in enumerate(tokens):
            if i == len(tokens) - 1:
                self._update(token, End())
            else:
                self._update(token, tokens[i + 1])

    def _parse_source(self, source: str) -> list[Token]:
        return [Word(w) for w in source.split() if not (w.startswith("<@") and w.endswith(">"))]

    def get_edges(self, token: str) -> Optional[dict[str, int]]:
        edges = self.edges.get(Word(token))
        if edges is None:
            return None
        else:
            return edges.to_tokens

    def learn(self, source: str):
        tokens = self._parse_source(source.lower())
        self._learn_from_tokens(tokens)

    def _pick_next(self, current_token: Token, allow_end: bool) -> Token:
        transitions = self.edges.get(current_token)
        if transitions is None:
            return End()
        else:
            next = transitions.pick_token(allow_end)
            if next is None:
                return End()
            else:
                return next
            
    def _join_tokens(self, tokens: list[Token]) -> str:
        buf = []
        for (i, c) in enumerate(tokens):
            match c:
                case End():
                    pass
                case Word(text):
                    buf.append(text + " " if i < len(tokens) - 1 else text)
        return "".join(buf)

            
    def _chain_tokens(self, starting_token: Optional[Token] = None, max_length: int = 20) -> list[Token]:
        
        tokens = []

        if starting_token is None:
            keys = self.edges.keys()
            if len(keys) == 0:
                return []
            else:
                starting_token = random.choice(list(keys))
                tokens.append(starting_token)
        
        current_token = starting_token

        while len(tokens) < max_length:
            next_token = self._pick_next(current_token, len(tokens) > 2)
            match next_token:
                case End():
                    break
                case token:
                    tokens.append(token)
            current_token = next_token

        return tokens
    
    def _chain(self, starting_token: Optional[Token] = None, max_length: int = 20) -> str:
        tokens = self._chain_tokens(starting_token, max_length)
        joined = self._join_tokens(tokens)
        return joined
    
    def respond(self, message: str, max_length: int = 20) -> str:
        tokens = self._parse_source(message)
        tt = [x for x in filter(token_is_word, tokens)]
        if len(tt) != 0 and tt[-1] in self.edges.keys():
            return self._chain(tt[-1], max_length=max_length)
        else:
            return self._chain(None, max_length=max_length)

__all__ = (MarkovChain,)
