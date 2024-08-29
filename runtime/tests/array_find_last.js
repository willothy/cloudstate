const baseArray = [5, 12, 8, 130, 44, 5];

{
    const object = {
        value: baseArray,
    };
    setRoot("test-root", object);
    commit();
}

{
    const object = getRoot("test-root");
    const found1 = object.value.findLast((element) => element > 10);
    if (found1 !== 44) {
        throw new Error(`Expected 44, got ${found1}`);
    }
    const found2 = object.value.findLast((element) => element > 200);
    if (found2 !== undefined) {
        throw new Error(`Expected undefined, got ${found2}`);
    }
    const found3 = object.value.findLast((element) => element <= 130);
    if (found3 !== 5) {
        throw new Error(`Expected 5, got ${found3}`);
    }
    const found4 = object.value.findLast((element) =>
        element > 15 && element < 100
    );
    if (found4 !== 44) {
        throw new Error(`Expected 44, got ${found4}`);
    }
    const found5 = object.value.findLast((element) => element === 5);
    if (found5 !== 5) {
        throw new Error(`Expected 5, got ${found5}`);
    }
}