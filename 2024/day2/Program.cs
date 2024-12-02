int numSafe = 0;

foreach (var line in File.ReadAllLines("input.txt"))
{
    if (line.Length == 0)
        continue;
    int[] numbers = line.Split(' ').Select(str => int.Parse(str)).ToArray();
    if (numbers.Length < 2)
        throw new Exception("not enough numbers");
    bool isSafe = true;
    bool isIncreasing = numbers[1] > numbers[0];
    for (int i = 1; i < numbers.Length; i++)
    {
        int diff = numbers[i] - numbers[i - 1];
        bool increased = diff > 0;
        if (diff == 0 || Math.Abs(diff) > 3 || increased != isIncreasing)
        {
            isSafe = false;
            break;
        }
    }

    if (isSafe)
    {
        numSafe++;
    }
}

Console.WriteLine($"part1: {numSafe}");
