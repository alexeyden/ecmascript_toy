var print_array = function(array) {
    var i = 0;
    while (i < array.length) {
	std.io.print(array[i], '');
	i = i + 1;
    }
    std.io.println();
};

var part = function(array, a, b) {
    var p = array[a];

    while (a <= b) {
	while (array[a] < p) a = a + 1;
	while (array[b] > p) b = b - 1;

	if (a <= b) {
	    var tmp = array[a];
	    array[a] = array[b];
	    array[b] = tmp;
	    
	    a = a + 1; b = b - 1;
	}
    }

    return a;
};

var qsort = function(array, a, b) {
    if (a >= b)
	return array;

    var p = part(array, a, b);
    
    qsort(array, a, p-1);
    qsort(array, p, b);
    return array;
};

var array = [4, 6, 1, 2, 0, 2, 8, 3];
qsort(array, 0, array.length-1);

print_array(array);

