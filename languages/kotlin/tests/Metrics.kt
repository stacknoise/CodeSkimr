package metrics

class M(private val items: List<Int>) {
    fun plain(): Int = 1

    fun branches(a: Int, b: Int): Int {
        if (a > 0 && b > 0) {
            return 1
        } else if (a < 0 || b < 0) {
            return 2
        }
        val c = a ?: b
        return if (a > b) a else b
    }

    fun loops(xs: IntArray) {
        for (x in xs) {
        }
        var i = 0
        while (i < 3) {
            i++
        }
        do {
            i--
        } while (i > 0)
    }

    fun cases(k: Int): String {
        return when (k) {
            1, 2 -> "a"
            in 3..4 -> "b"
            else -> "c"
        }
    }

    fun guards(k: Int): String = when {
        k > 1 -> "a"
        else -> "b"
    }

    fun catches() {
        try {
            plain()
        } catch (e: IllegalStateException) {
        } catch (e: Exception) {
        } finally {
            plain()
        }
    }

    fun outer(): Int {
        val f = { x: Int -> if (x > 0) 1 else 2 }
        return f(1)
    }

    fun inner(): Int {
        fun local(y: Int) = if (y > 0) y else 0
        return local(1)
    }

    constructor(n: Int) : this(listOf(n)) {
        println(n)
    }

    abstract fun noBody()
}

interface Shape {
    fun area(): Double
}
