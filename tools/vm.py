#!/bin/env python3

import struct
import sys

from enum import Enum
from collections import namedtuple

class VirtualMachine:
  class Command(Enum):
    PUSH_FLOAT = 0x20
    PUSH_STR   = 0x21
    PUSH_INT   = 0x22
    PUSH_FN    = 0x23
    TAKE       = 0x24
    SWAP       = 0x25
    POP        = 0x26

    LOAD       = 0x31
    STORE      = 0x32

    JUMPIF     = 0x40
    JUMP       = 0x41
    CALL       = 0x42

    ADD        = 0x50
    SUB        = 0x51
    MUL        = 0x52
    DIV        = 0x53
    MOD        = 0x54
    NEG        = 0x55

    LT         = 0x60
    GT         = 0x61
    EQ         = 0x62
    NEQ        = 0x63
    LEQ        = 0x64
    GEQ        = 0x65
    AND        = 0x66
    OR         = 0x67
    NOT        = 0x68

    GET        = 0x70
    PUSH_DICT  = 0x71
    PUSH_ARRAY = 0x72

  class Type(Enum):
    UNDEF  = 0x00
    INT    = 0x01
    FLOAT  = 0x02
    STR    = 0x03
    REF    = 0x04
    FN     = 0x05
    DICT   = 0x06
    NATIVE = 0x07

  class Reference:
    def __init__(self, base_ptr, target_ptr, target_name):
      self.base_ptr = base_ptr
      self.target_ptr = target_ptr
      self.target_name = target_name

    def __add__(self, other):
      if type(other) is int:
        return self.__class__(self.base_ptr, self.target_ptr + other, self.target_name)
      elif type(other) is type(self):
        return self.__class__(self.base_ptr, self.target_ptr + other.target_ptr, self.target_name)
      else:
        raise TypeError("expected int or reference")
        
    def __repr__(self):
      return self.__str__()

    def __str__(self):
      if self.base_ptr is None:
        return f'&({self.target_ptr})'
      else:
        return f'&({self.base_ptr}@{self.target_name}=>{self.target_ptr})' 

  class Function:
    def __init__(self, start, env_frames, frame_size):
      self.start = start
      self.env_frames = env_frames
      self.frame_size = frame_size

    def __str__(self):
      return f'fn@{self.start} (env={len(self.env_frames)} fr={self.frame_size})' 

  class Value:
    def __init__(self, t, v):
      self.type = t
      self.value = v

    def __repr__(self):
      return self.__str__()

    def __str__(self):
      return f'{self.type.name} {self.value}'

  def __init__(self, data, debug = False):
    self.data = data
    self.debug = debug

    self.ip = 0
    self.offset = 0

    self.heap = []
    self.stack = []

    std = {
      'io': self.Value(self.Type.DICT, {
        'println' : self.Value(self.Type.NATIVE, lambda *args: print(*args)),
        'print'   : self.Value(self.Type.NATIVE, lambda *args: print(*args, end=''))
      }),
      'sys': self.Value(self.Type.DICT, {
        'exit'    : self.Value(self.Type.NATIVE, lambda *_: sys.exit(0))
      })
    }

    self.heap.append(self.Value(self.Type.REF, self.Reference(base_ptr = None, target_name = None, target_ptr = 1)))
    self._init_builtin(std)
    
  def _init_builtin(self, obj):
    heap_obj = {}
    self.heap.append(self.Value(self.Type.DICT, heap_obj))
    self.heap.append(self.Value(self.Type.REF,
                                self.Reference(base_ptr = None, target_name = None, target_ptr = len(self.heap)-1)))
    base_ptr = len(self.heap) - 1

    for name, value in obj.items():
      if value.type == self.Type.DICT:
        target_ptr = self._init_builtin(value.value)
      else:
        self.heap.append(value)
        target_ptr = len(self.heap) - 1

      ref = self.Reference(
        base_ptr = base_ptr,
        target_ptr = target_ptr,
        target_name = name)

      heap_obj[name] = self.Value(self.Type.REF, ref)

    return base_ptr
                 
  def step(self):
    self._next_cmd()

  def run(self):
    while self.offset < len(self.data):
      self.step()

  def run_steps(self, n):
    for _ in range(0, n):
      self.step()

  def _next_cmd(self):
    if self.offset >= len(self.data):
      return

    if self.debug:
      print(f'{self.ip:04} {self.offset:08}: ', end='')

    cmd = struct.unpack_from("<b", self.data, self.offset)[0]

    self.ip += 1
    self.offset += 1

    if self.Command.PUSH_FLOAT.value <= cmd <= self.Command.POP.value:
      self._handle_stack(self.Command(cmd))
    elif self.Command.JUMPIF.value <= cmd <= self.Command.CALL.value:
      self._handle_control(self.Command(cmd))
    elif self.Command.LOAD.value <= cmd <= self.Command.STORE.value:
      self._handle_mem(self.Command(cmd))
    elif self.Command.ADD.value <= cmd <= self.Command.NEG.value:
      self._handle_math(self.Command(cmd))
    elif self.Command.LT.value <= cmd <= self.Command.NOT.value:
      self._handle_logic(self.Command(cmd))
    elif self.Command.GET.value <= cmd <= self.Command.PUSH_ARRAY.value:
      self._handle_dict(self.Command(cmd))
    else:
      raise Exception(f'Unknown opcode: {cmd}')  

  def _read_arg_f32(self):
    arg = struct.unpack_from("<f", self.data, self.offset)
    self.offset += 4
    return arg[0]

  def _read_arg_u32(self):
    arg = struct.unpack_from("<i", self.data, self.offset)
    self.offset += 4
    return arg[0]

  def _read_arg_str(self):
    length = self._read_arg_u32()
    arg = struct.unpack_from(f"<{length}s", self.data, self.offset)
    self.offset += len(arg[0])
    return arg[0].decode('utf-8')

  def _print_cmd(self, cmd, direct_args, stack_args, result = None):
    def format_args(args):
      if type(args) == dict:
        return ", ".join([f"{name} = {arg}" for name, arg in args.items()])
      elif type(args) == list:
        return ", ".join([f"{arg}" for arg in args])
      else:
        raise Exception("Wrong argument")
      
    if self.debug:
      direct = format_args(direct_args)
      stack = format_args(stack_args) 
      result = f" => {format_args(result)}" if result else ""
      print(f"{cmd.name} ({direct}) [{stack}] {result}")

  def _handle_stack(self, cmd):
    if cmd == self.Command.PUSH_FLOAT:
      arg = self._read_arg_f32()
      val = self.Value(self.Type.FLOAT, arg)
      self.stack.append(val)

      self._print_cmd(cmd, direct_args=[val], stack_args=[])

    elif cmd == self.Command.PUSH_STR:
      arg = self._read_arg_str()
      str_val = self.Value(self.Type.STR, arg) 
      self.stack.append(str_val)

      self._print_cmd(cmd, direct_args=[str_val], stack_args=[])

    elif cmd == self.Command.PUSH_INT:
      arg = self._read_arg_u32()
      val = self.Value(self.Type.INT, arg)
      self.stack.append(val)

      self._print_cmd(cmd, direct_args=[val], stack_args=[])

    elif cmd == self.Command.PUSH_FN:
      fr_count  = self._read_arg_u32() 
      fr_offset = self._read_arg_u32() 
      fr_size   = self._read_arg_u32() 

      frames = self.stack[-fr_offset-1:-fr_offset-1 + fr_count]
      addr = self.stack.pop();

      fn = self.Value(self.Type.FN, self.Function(addr.value, frames, fr_size))
      self.stack.append(fn)

      self._print_cmd(cmd, direct_args={
        'fr_count' : self.Value(self.Type.INT, fr_count),
        'fr_offset': self.Value(self.Type.INT, fr_offset),
        'fr_size'  : self.Value(self.Type.INT, fr_size),
        }, stack_args = [addr])

    elif cmd == self.Command.TAKE:
      offset = self._read_arg_u32() 

      val = self.stack[-offset-1]
      self.stack.append(val) 

      self._print_cmd(cmd,
                      direct_args=[self.Value(self.Type.INT, offset)],
                      stack_args=[],
                      result=[val])
    elif cmd == self.Command.SWAP:
      a = self._read_arg_u32() 
      b = self._read_arg_u32() 
      
      self.stack[-a-1], self.stack[-b-1] = self.stack[-b-1], self.stack[-a-1]

      self._print_cmd(cmd,
                      direct_args=[self.Value(self.Type.INT, a),
                                   self.Value(self.Type.INT, b)],
                      stack_args=[])
    elif cmd == self.Command.POP:
      n = self._read_arg_u32()
      self.stack = self.stack[:-n]

      self._print_cmd(cmd,
                      direct_args=[self.Value(self.Type.INT, n)],
                      stack_args=[])

  def _handle_mem(self, cmd):
    if cmd == self.Command.LOAD:
      offset = self._read_arg_u32()
      addr = self.stack[-1]

      if type(addr.value) is self.Reference:
        value = self.heap[int(addr.value.target_ptr) + int(offset)]
      else:
        value = self.heap[int(addr.value)+int(offset)]

      self._print_cmd(cmd,
                      direct_args=[self.Value(self.Type.INT, offset)],
                      stack_args=[addr],
                      result=[value])

      self.stack[-1] = value

    elif cmd == self.Command.STORE:
      addr = self.stack.pop()
      value = self.stack.pop()

      self._print_cmd(cmd, direct_args=[], stack_args={'addr': addr, 'value': value})

      if addr.value.target_ptr is None:
        self.heap.append(value)

        value = self.Value(self.Type.REF, self.Reference(
          base_ptr = addr.value.base_ptr,
          target_name = addr.value.target_name,
          target_ptr = len(self.heap) - 1))

        obj = self.heap[addr.value.base_ptr].value
        obj[addr.value.target_name] = value 
      else:
        self.heap[int(addr.value.target_ptr)] = value

  def _handle_control(self, cmd):
    if cmd == self.Command.JUMPIF:
      addr = self.stack.pop()
      cond = self.stack.pop()

      if cond.value:
        self.offset = addr.value

      self._print_cmd(cmd,
                      direct_args=[],
                      stack_args={'addr': addr, 'cond': cond},
                      result=["JUMP" if cond else "PASS"])

    elif cmd == self.Command.JUMP:
      addr = self.stack.pop()
      self.offset = addr.value

      self._print_cmd(cmd,
                      direct_args=[],
                      stack_args=[addr])

    elif cmd == self.Command.CALL:
      fn_ref = self.stack.pop();
      fn = self.heap[fn_ref.value.target_ptr] if fn_ref.type == self.Type.REF else fn_ref
      n_args = self.stack.pop();

      args = []

      if fn.type == self.Type.NATIVE:
        for _ in range(0, n_args.value):
          args.insert(0, self.stack.pop().value)
        fn.value(*args)
        ip = self.stack.pop().value
        self.offset = ip
        self.stack.append(self.Value(self.Type.UNDEF, 0))

      elif fn.type == self.Type.FN:
        self.heap += [self.Value(self.Type.UNDEF, 0)]*fn.value.frame_size

        for a in range(0, n_args.value):
          arg = self.stack.pop()
          self.heap[-fn.value.frame_size + a] = arg
          args.insert(0, arg.value)

        target_ptr = len(self.heap) - fn.value.frame_size
        value = self.Value(self.Type.REF, self.Reference(base_ptr = None, target_name = None, target_ptr = target_ptr))

        base_ptr = fn_ref.value.base_ptr if fn_ref.type == self.Type.REF else None 
        self.heap[-fn.value.frame_size + n_args.value] = self.Value(self.Type.REF,
                                                                    self.Reference(base_ptr = "this",
                                                                                   target_name = "this",
                                                                                   target_ptr = base_ptr))
        self.stack.append(value)
        self.stack += fn.value.env_frames
        self.offset = fn.value.start

      self._print_cmd(cmd,
                      direct_args=[],
                      stack_args={'fn': fn, 'n_args': n_args, 'args': args})

  def _handle_math(self, cmd):
    def handle_binary(op):
      v1 = self.stack.pop()
      v2 = self.stack.pop()
      v = op(v2.value, v1.value)

      if type(v) == str:
        t = self.Type.STR
      elif type(v) == int:
        t = self.Type.INT
      elif type(v) == float:
        t = self.Type.FLOAT
      elif type(v) == self.Reference:
        t = self.Type.REF
      else:
        raise TypeError('wrong type for a math')
          
      self.stack.append(self.Value(t, v))

      self._print_cmd(cmd, direct_args=[], stack_args=[v1, v2], result=[self.stack[-1]])

    if cmd == self.Command.ADD:
      handle_binary(lambda a,b: a+b)
    elif cmd == self.Command.SUB:
      handle_binary(lambda a,b: a-b)
    elif cmd == self.Command.MUL:
      handle_binary(lambda a,b: a*b)
    elif cmd == self.Command.DIV:
      handle_binary(lambda a,b: a/b)
    elif cmd == self.Command.MOD:
      handle_binary(lambda a,b: a%b)
    elif cmd == self.Command.NEG:
      v1 = self.stack.pop()
      self._print_cmd(cmd, direct_args=[], stack_args=[v1])
      self.stack.append(self.Value(v1.type, -v1.value))

  def _handle_logic(self, cmd):
    def handle_binary(op):
      v1 = self.stack.pop()
      v2 = self.stack.pop()
      v = op(v2.value, v1.value)
      self.stack.append(self.Value(self.Type.FLOAT, v))

      self._print_cmd(cmd, direct_args=[], stack_args=[v1, v2], result=[self.stack[-1]])
    
    if cmd == self.Command.LT:
      handle_binary(lambda a,b: a < b)
    elif cmd == self.Command.GT:
      handle_binary(lambda a,b: a > b)
    elif cmd == self.Command.EQ:
      handle_binary(lambda a,b: a == b)
    elif cmd == self.Command.NEQ:
      handle_binary(lambda a,b: a != b)
    elif cmd == self.Command.LEQ:
      handle_binary(lambda a,b: a <= b)
    elif cmd == self.Command.GEQ:
      handle_binary(lambda a,b: a >= b)
    elif cmd == self.Command.AND:
      handle_binary(lambda a,b: a and b)
    elif cmd == self.Command.OR:
      handle_binary(lambda a,b: a or b)
    elif cmd == self.Command.NOT:
      self._print_cmd(cmd, direct_args=[], stack_args=[self.stack[-1]])
      self.stack[-1].value = not self.stack[-1].value

  def _handle_dict(self, cmd):
    if cmd == self.Command.GET:
      key = self.stack.pop()
      d = self.stack.pop()
      ref = self.Reference(base_ptr = d.value.target_ptr, target_name = key.value, target_ptr = None)

      target_dict = self.heap[d.value.target_ptr].value

      if key.value == "length":
        self.heap.append(self.Value(self.Type.FLOAT, len(target_dict)))
        ref.target_ptr = len(self.heap) - 1

      elif key.value in target_dict:
        ref.target_ptr = target_dict[key.value].value.target_ptr
        
      v = self.Value(self.Type.REF, ref)
      self.stack.append(v)

      self._print_cmd(cmd, direct_args=[], stack_args={'dict': d, 'value': key}, result=[v])

    elif cmd == self.Command.PUSH_DICT:
      length = self._read_arg_u32()

      new_dict = self.Value(self.Type.DICT, {})
      self.heap.append(new_dict)

      dict_ptr = len(self.heap) - 1

      for i in range(0, length):
        value = self.stack.pop()
        key = self.stack.pop()
        ref = self.Reference(base_ptr = dict_ptr, target_name = key.value, target_ptr = len(self.heap))
        self.heap.append(value)
        new_dict.value[key.value] = self.Value(self.Type.REF, ref)

      ref = self.Reference(base_ptr = None, target_name = None, target_ptr = dict_ptr)
      self.stack.append(self.Value(self.Type.REF, ref))

      self._print_cmd(cmd,
                      direct_args=[self.Value(self.Type.INT, length)],
                      stack_args=list(new_dict.value.values()))
      
    elif cmd == self.Command.PUSH_ARRAY:
      length = self._read_arg_u32()

      new_dict = self.Value(self.Type.DICT, {})

      self.heap.append(new_dict)
      dict_ptr = len(self.heap) - 1

      for i in range(0, length):
        item = self.stack.pop()
        ref = self.Reference(base_ptr = dict_ptr, target_name = i, target_ptr = len(self.heap))
        self.heap.append(item)
        new_dict.value[length - 1 - i] = self.Value(self.Type.REF, ref)

      ref = self.Reference(base_ptr = None, target_name = None, target_ptr = dict_ptr) 
      self.stack.append(self.Value(self.Type.REF, ref))

      self._print_cmd(cmd,
                      direct_args=[self.Value(self.Type.INT, length)],
                      stack_args=list(new_dict.value.values()))
class Main:
  def __init__(self, path, debug):
    with open(path, 'rb') as f:
      self.data = f.read()
    self.interpreter = VirtualMachine(self.data)
    self.interpreter.debug = debug

  def run(self):
    self.interpreter.run()

  def repl(self):

    while True:
      print('> ', end='')
      cmd = input()

      if cmd == 'n' or cmd == 'next':
        self.interpreter.step()
      elif cmd == 'r' or cmd == 'run':
        self.interpreter.run()
      elif cmd == 'q' or cmd == 'quit':
        break
      elif cmd == 's' or cmd == 'stack':
        for i, v in enumerate(self.interpreter.stack):
          print(f"{i:04} {v.type:<15} {v.value}")
      elif cmd == 'm' or cmd == 'mem':
        for i, v in enumerate(self.interpreter.heap):
          print(f"{i:04} {v.type:<15} {v.value}")
      elif cmd == '?' or cmd == 'status':
        print('ip = {}, sp = {}'.format(self.interpreter.offset, len(self.interpreter.stack)))
      elif cmd.startswith('k ') and (cmd.split()[0] == 'k' or cmd.split()[0] == 'skip'):
        n = int(cmd.split()[1])
        self.interpreter.run_steps(n)
      else:
        print('wut?')

if __name__ == "__main__":
  if sys.argv[1] == "-r":
    Main(path=sys.argv[2], debug=True).repl()
  else:
    Main(path=sys.argv[1], debug=False).run()

