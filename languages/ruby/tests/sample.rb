# Shop module.
module Shop
  # An order.
  class Order < Base
    def initialize(id)
      @id = id
    end

    # Total of the order.
    def total
      if @id > 0 && @id < 5
        @id
      elsif @id
        1
      end
    end

    def self.build(id)
      new(id)
    end
  end
end

def helper(x)
  x.each { |y| puts y } while false
end

if defined?(Gem)
  def with_gem
    1
  end
else
  def without_gem
    2
  end
end

begin
  def guarded
    3
  end
rescue StandardError
  nil
end

class Late
  unless false
    def conditional
      4
    end
  end
end
