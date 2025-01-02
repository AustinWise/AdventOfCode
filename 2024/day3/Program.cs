
using System.Globalization;
using System.Text.RegularExpressions;

partial class Program
{
    static void Main()
    {
        string input = File.ReadAllText("input.txt");
        long part1 = 0;
        foreach (Match m in MulRegex().Matches(input))
        {
            int x = int.Parse(m.Groups["x"].Value, CultureInfo.InvariantCulture);
            int y = int.Parse(m.Groups["y"].Value, CultureInfo.InvariantCulture);
            part1 += x * y;
        }
        Console.WriteLine($"Part 1: {part1}");
    }

    [GeneratedRegex(@"mul\((?<x>\d+),(?<y>\d+)\)")]
    private static partial Regex MulRegex();
}
