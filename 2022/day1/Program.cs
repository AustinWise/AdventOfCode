int max = 0;
int cur = 0;
foreach (var line in day1.Properties.Resources.Input.Split('\n').Select(l => l.Trim('\r')))
{
    if (line.Length == 0)
    {
        max = Math.Max(cur, max);
        cur = 0;
    }
    else
    {
        cur += int.Parse(line);
    }
}
Console.WriteLine(max);
