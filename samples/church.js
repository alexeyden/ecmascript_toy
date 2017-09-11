var zero = function(f) {
    return function(x) {
	return x;
    };
};

var next = function(p) {
    return function(f) {
	return function(x) {
	    return f(p(f)(x));
	};
    };
};

var church = function(n, x) {
    if (n == 0)
	return x;
    else
	return church(n - 1, next(x));
};

var plus = function (x) { return x + 1; };

std.io.println(church(5, zero)(plus)(0));

