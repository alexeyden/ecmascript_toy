var x  = {
    hello: [0,1,2,3,{a: 1, b: 2, test: 3}],
    world: 7
};
var y = x.hello[1+3]['test'];

std.io.println('x', x.world, '( = 7)');
std.io.println('y', y, '( = 3)');

