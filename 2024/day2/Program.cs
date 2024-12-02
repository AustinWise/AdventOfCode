int numSafe = 0;
int numSafeAfterUsingProblemDampener = 0;

static bool IsSafe(int[] numbers)
{
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

    return isSafe;
}

foreach (var line in File.ReadAllLines("input.txt"))
{
    if (line.Length == 0)
        continue;
    int[] numbers = line.Split(' ').Select(str => int.Parse(str)).ToArray();
    if (numbers.Length < 2)
        throw new Exception("not enough numbers");
    bool isSafe = IsSafe(numbers);

    if (isSafe)
    {
        numSafe++;
        numSafeAfterUsingProblemDampener++;
    }
    else
    {
        // Check if removing a single element would make the report safe.
        // Maybe there is a way to not brute force this...
        int[] fixedNumbers = new int[numbers.Length - 1];

        for (int i = 0; i < numbers.Length; i++)
        {
            if (i > 0)
            {
                numbers.AsSpan().Slice(0, i).CopyTo(fixedNumbers);
            }
            if (i < numbers.Length - 1)
            {
                numbers.AsSpan(i + 1, numbers.Length - i - 1).CopyTo(fixedNumbers.AsSpan().Slice(i));
            }
            if (IsSafe(fixedNumbers))
            {
                numSafeAfterUsingProblemDampener++;
                break;
            }
        }
    }
}

Console.WriteLine($"part1: {numSafe}");
Console.WriteLine($"part2: {numSafeAfterUsingProblemDampener}");
