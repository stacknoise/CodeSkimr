package refs

import com.example.util.Helper
import com.example.util.Tools as T
import com.example.util.*

open class Base
interface Walker
interface Talker

class Dog(val name: String, private val owner: Owner?) : Base(), Walker, Talker by delegate {
    private val helper: Helper = Helper()
    var count = 0

    override fun walk() {
        val x: Count = 1
        val y = Helper()
        var total = count + 1
        total += helper.size
        count = total
        count++
        helper.run(x, y)
        this.bark(total)
        super.walk()
        T.log(name)
        val list: List<Item> = listOf()
        for (item in list) {
            handle(item)
        }
        val f = { z: Int -> z + 1 }
        if (owner is Owner) {
            println(owner.name)
        }
        val o = other as Other
        val cls = Dog::class
        val ref = ::handle
    }

    fun bark(n: Int): Result<Sound> {
        throw IllegalStateException("no")
    }

    @Deprecated("old")
    fun handle(item: Item) {}
}
