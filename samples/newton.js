var x0 = 9;
var x = x0;
var i = 0;
var imax = 1000;
var xp = x - x/2;

// comment

while (i < imax) {
    x = x - (x*x - x0)/(2 * x); // inline comment
    i = i + 1;
}

// concatenation check

var square = 'Square';
var root_of = 'root of';
var msg = square + ' ' + root_of;

std.io.println(msg, x0, 'is', x);

