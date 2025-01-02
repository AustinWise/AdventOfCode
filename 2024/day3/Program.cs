
using System.Globalization;
using System.Text.RegularExpressions;

partial class Program
{
    static void Main()
    {
        string input = File.ReadAllText("input.txt");
        bool enabled = true;
        long part1 = 0;
        long part2 = 0;
        foreach (Match m in MulRegex().Matches(input))
        {
            if (m.Value == "do()")
            {
                enabled = true;
            }
            else if (m.Value == "don't()")
            {
                enabled = false;
            }
            else
            {
                int x = int.Parse(m.Groups["x"].Value, CultureInfo.InvariantCulture);
                int y = int.Parse(m.Groups["y"].Value, CultureInfo.InvariantCulture);
                part1 += x * y;
                if (enabled)
                    part2 += x * y;
            }
        }
        Console.WriteLine($"Part 1: {part1}");
        Console.WriteLine($"Part 2: {part2}");
    }

    [GeneratedRegex(@"do\(\)|don\'t\(\)|mul\((?<x>\d+),(?<y>\d+)\)")]
    private static partial Regex MulRegex();
}
