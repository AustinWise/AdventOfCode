using System.Diagnostics;

var leftList = new List<int>();
var rightList = new List<int>();

foreach (var line in File.ReadAllLines("input.txt"))
{
    if (line.Length == 0)
        continue;
    string[] numberStrs = line.Split(' ', StringSplitOptions.RemoveEmptyEntries);
    if (numberStrs.Length != 2)
        throw new Exception("unpexted length");
    leftList.Add(int.Parse(numberStrs[0]));
    rightList.Add(int.Parse(numberStrs[1]));
}

Debug.Assert(leftList.Count == rightList.Count);

leftList.Sort();
rightList.Sort();

int sumOfDistances = 0;
for (int i = 0; i < leftList.Count; i++)
{
    sumOfDistances += Math.Abs(leftList[i] - rightList[i]);
}

Console.WriteLine($"part1: {sumOfDistances}");

// We want to find the number of times each element in the left list appears in the right.
// Since we already sorted the right list, we could walk both lists at the same time, maintaining two indexes.
// But that sounds annoying, so just count everything in right side first then look up the count for each in the left.

var rightCounts = rightList.GroupBy(i => i).ToDictionary(g => g.Key, g => g.Count());

int partTwoValue = leftList.Select(i => rightCounts.TryGetValue(i, out int value) ? value * i : 0).Sum();
Console.WriteLine($"part2: {partTwoValue}");
