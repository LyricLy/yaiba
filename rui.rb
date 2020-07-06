

# returns [num, offset]
def parse_num(input, pos)
    offset = 0
    num = ''
    while input[pos + offset].match? /\d/
        num << input[pos + offset]
        offset += 1
    end
    if num.size == 0
        raise "Expected number after '#{input[pos-1]}'"
    end
    return num.to_i, offset
end

def read
    val = STDIN.gets.chomp
    puts "Reading: #{val}"
    return val.to_i
end

def write(val)
    # TODO
    puts val.to_s
end



input = File.exists?(ARGV[0]) ? File.open(ARGV[0], 'r').read : ARGV[0]

input = input.gsub(/ |\#.*/, '') # remove whitespace and comments

# store the start of each line
lines = [0]
0.upto(input.length) do |i|
    if input[i] == "\n"
        lines.push(i + 1)
    end
end

# each thread is [codepos, value]
POS = 0
VAL = 1
FRESH = 2
threads = [
    [0, 0, false]
]

loop do
    break if threads.size == 0

    i = 0
    created = 0
    while i < threads.size

        thread = threads[i]

        if thread[FRESH]
            thread[FRESH] = false
            i += 1
            next
        end

        instr = input[thread[POS]]
        thread[POS] += 1

        case instr
        when '='
            thread[VAL], off = parse_num(input, thread[POS])
            thread[POS] += off
        when '+'
            line, off = parse_num(input, thread[POS])
            thread[POS] += off
            raise "Line number out of bounds: #{line}" if line <= 0 || line > lines.size
            pos = lines[line - 1]
            threads << [pos, 0, true]
            created += 1
        when '*'
            line, off = parse_num(input, thread[POS])
            thread[POS] += off
            raise "Line number out of bounds: #{line}" if line <= 0 || line > lines.size
            pos = lines[line - 1]
            thread[VAL].times do
                threads << [pos, 0, true]
            end
            created += thread[VAL]
        when '-'
            num, off = parse_num(input, thread[POS])
            thread[POS] += off
            j = 0
            count = 0
            while j < threads.size 
                thread2 = threads[j]
                if thread2[VAL] == num && i != j
                    i -= 1 if i > j
                    threads.delete_at(j)
                    count += 1
                else
                    j += 1
                end
            end
            thread[VAL] = count
        when 'r'
            thread[VAL] = read
        when 'w'
            write thread[VAL]
        when '!'
            threads.delete_at(i)
            i -= 1
        when '.'
            # sleep
        when '$'
            threads.each_with_index do |thread2, index|
                thread2[VAL] += thread[VAL] if index != i
            end
        when '~'
            threads.each_with_index do |thread2, index|
                thread2[VAL] -= thread[VAL] if index != i
                thread2[VAL] = 0 if thread2[VAL] < 0
            end
        when ':'
            line, off = parse_num(input, thread[POS])
            thread[POS] += off
            raise "Line number out of bounds: #{line}" if line <= 0 || line > lines.size
            pos = lines[line - 1]
            thread[POS] = pos
        when "\n"
            i -= 1 # ignore
        else raise "Unknown symbol '#{instr}'"
        end

        i += 1
    end
end
