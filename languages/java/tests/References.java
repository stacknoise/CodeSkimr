package com.example;

import java.util.List;
import java.util.Map;
import java.io.*;
import static java.lang.Math.max;

class Child extends Parent<String> implements Runnable, Comparable<Child> {
    private Helper helper = new Helper();
    private int total;

    @Override
    public void run() {
        helper.start();
        this.total += max(1, 2);
        super.run();
        new Helper().stop();
        total++;
    }

    int use(Helper other, int count) throws IOException {
        Helper local = new Helper();
        var made = new Helper<String>();
        for (Helper each : List.of(other)) {
            each.start();
        }
        try {
            count = count + total;
        } catch (IOException | RuntimeException e) {
            throw e;
        }
        Runnable r = () -> local.start();
        Object o = (Object) other;
        return count > 0 ? total : count;
    }
}

interface Shape extends Comparable<Shape>, java.io.Serializable {}
