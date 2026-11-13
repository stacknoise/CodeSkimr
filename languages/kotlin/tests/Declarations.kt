package com.example.shapes

import com.example.base.Base
import com.example.base.Named as Label

/** A plain class with a primary constructor. */
open class Shape(val id: Int, private var label: String = "none") : Base(), Named {
    /** Companion with a name. */
    companion object Factory {
        /** Maximum size. */
        const val MAX_SIZE = 10
        private val registry = mutableMapOf<Int, Shape>()
        fun create(id: Int): Shape = Shape(id)
    }

    companion object {
        const val UNNAMED = 1
    }

    /** The area. */
    open fun area(): Double = 0.0

    protected fun helper() {
        val local = 1
        fun inner() = local
    }

    internal var count: Int = 0
        get() = field
        private set

    lateinit var late: String

    val Int.double: Int get() = this * 2

    constructor(label: String) : this(0, label)

    /** Documented constructor. */
    private constructor(id: Int, other: Shape) : this(id)

    init {
        println("created")
    }

    enum class Kind(val sides: Int) {
        TRIANGLE(3),
        SQUARE(4) {
            override fun describe() = "square"
        },
        ;

        open fun describe() = "kind"
    }

    object Singleton {
        fun hello() = "hi"
    }

    private class Hidden
}

/**
 * An interface.
 *
 * @property size the size
 */
interface Named {
    val name: String
    /** Greets. */
    fun greet(): String
    fun shout(): String = greet().uppercase()
}

fun interface Action {
    fun run()
}

sealed interface Result
data class Success(val value: Int) : Result
data class Failure(val reason: String, internal val code: Int) : Result
annotation class Marker
enum class Color { RED, GREEN }
typealias Handler = (Int) -> Unit

/** Shouts. */
fun String.shout(): String = this.uppercase()
internal fun top() = Shape(1)
private fun hidden() {}
const val TOP_LEVEL = 3
val topProperty = 4
