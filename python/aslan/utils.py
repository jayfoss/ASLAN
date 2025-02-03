from typing import TypeVar, Any
from dataclasses import dataclass
import random

@dataclass
class StackItem:
    source: Any
    target: Any 
    is_array: bool

T = TypeVar('T')

def deep_copy(source: T) -> T:
    if source is None or not isinstance(source, (dict, list)):
        return source

    stack = []
    is_array = isinstance(source, list)
    result = [] if is_array else {}

    stack.append(StackItem(
        source=source,
        target=result,
        is_array=is_array
    ))

    while stack:
        current = stack.pop()
        current_source = current.source
        current_target = current.target
        is_current_array = current.is_array

        keys = range(len(current_source)) if is_current_array else current_source.keys()

        for key in keys:
            value = current_source[key]

            if value is None or not isinstance(value, (dict, list)):
                if is_current_array:
                    while len(current_target) <= key:
                        current_target.append(None)
                    current_target[key] = value
                else:
                    current_target[key] = value
                continue

            is_value_array = isinstance(value, list)
            new_target = [] if is_value_array else {}
            if is_current_array:
                while len(current_target) <= key:
                    current_target.append(None)
                current_target[key] = new_target
            else:
                current_target[key] = new_target

            stack.append(StackItem(
                source=value,
                target=new_target,
                is_array=is_value_array
            ))

    return result

def generate_random_idempotency_key() -> str:
    return random.random().hex()[2:15] + random.random().hex()[2:15]