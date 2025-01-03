
using System.Diagnostics;

class Program
{
    static void Main()
    {
        TestCases();

        Console.WriteLine("part 1: {0}", CountXmases(File.ReadAllText("input.txt")));
        Console.WriteLine("part 2: {0}", CountX_mases(File.ReadAllText("input.txt")));
    }

    [Conditional("DEBUG")]
    static void TestCases()
    {
        const string sample = @"MMMSXXMASM
MSAMXMSMSA
AMXSXMAAMM
MSAMASMSMX
XMASAMXAMM
XXAMMXXAMA
SMSMSASXSS
SAXAMASAAA
MAMMMXMMMM
MXMXAXMASX";
        int part1 = CountXmases(sample);
        Debug.Assert(part1 == 18);
        int part2 = CountX_mases(sample);
        Debug.Assert(part2 == 9);
    }

    class Grid
    {
        private readonly char[,] _chars;

        public int Width { get; }

        public int Height { get; }

        public Grid(string input)
        {
            string[] lines = input.Split(['\r', '\n'], StringSplitOptions.RemoveEmptyEntries);
            if (!lines.All(l => l.Length == lines[0].Length))
            {
                throw new Exception("Not all lines the same length");
            }

            Height = lines.Length;
            Width = lines[0].Length;
            _chars = new char[Height, Width];
            for (int y = 0; y < Height; y++)
            {
                for (int x = 0; x < Width; x++)
                {
                    _chars[y, x] = lines[y][x];
                }
            }
        }

        public char Get(int x, int y)
        {
            if (x < 0 || x >= Width || y < 0 || y >= Height)
            {
                return '\0';
            }
            return _chars[y, x];
        }

        public int CountXmases()
        {
            // we look in the following directions:
            // * down
            // * right
            // * diagonally [1, 1]
            // * diagonally [1, -1]
            // we check for christmas forward and backwards in each direction.
            // we don't check in every direction for both forwards and backwards because that would double count.

            int count = 0;
            Span<char> buf = stackalloc char[4];
            for (int y = 0; y < Height; y++)
            {
                for (int x = 0; x < Width; x++)
                {
                    buf[0] = Get(x, y);
                    buf[1] = Get(x, y + 1);
                    buf[2] = Get(x, y + 2);
                    buf[3] = Get(x, y + 3);
                    if (IsXmas(buf))
                        count++;
                    buf[0] = Get(x, y);
                    buf[1] = Get(x + 1, y);
                    buf[2] = Get(x + 2, y);
                    buf[3] = Get(x + 3, y);
                    if (IsXmas(buf))
                        count++;
                    buf[0] = Get(x, y);
                    buf[1] = Get(x + 1, y + 1);
                    buf[2] = Get(x + 2, y + 2);
                    buf[3] = Get(x + 3, y + 3);
                    if (IsXmas(buf))
                        count++;
                    buf[0] = Get(x, y);
                    buf[1] = Get(x + 1, y - 1);
                    buf[2] = Get(x + 2, y - 2);
                    buf[3] = Get(x + 3, y - 3);
                    if (IsXmas(buf))
                        count++;
                }
            }

            static bool IsXmas(Span<char> buf)
            {
                return buf.SequenceEqual("XMAS".AsSpan()) || buf.SequenceEqual("SAMX".AsSpan());
            }

            return count;
        }

        public int CountX_mases()
        {
            int count = 0;


            for (int y = 0; y < Height; y++)
            {
                for (int x = 0; x < Width; x++)
                {
                    if (Get(x, y) != 'A')
                        continue;
                    if (!IsMas(Get(x - 1, y - 1), Get(x + 1, y + 1)))
                        continue;
                    if (!IsMas(Get(x - 1, y + 1), Get(x + 1, y - 1)))
                        continue;
                    count++;
                }
            }

            static bool IsMas(char ch1, char ch2)
            {
                return (ch1 == 'M' && ch2 == 'S') || (ch1 == 'S' && ch2 == 'M');
            }

            return count;
        }
    }

    static int CountXmases(string input)
    {
        return new Grid(input).CountXmases();
    }

    static int CountX_mases(string input)
    {
        return new Grid(input).CountX_mases();
    }
}
