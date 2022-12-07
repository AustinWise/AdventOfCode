
using System.Diagnostics;

int partOneScore = 0;
int partTwoScore = 0;
foreach (var line in day2.Properties.Resources.Input.Split(new char[] { '\r', '\n' }, StringSplitOptions.RemoveEmptyEntries))
{
    Debug.Assert(line.Length == 3);
    Debug.Assert(line[1] == ' ');

    // A for Rock, B for Paper, and C for Scissors
    char opponentChar = line[0];
    // X for Rock, Y for Paper, and Z for Scissors
    char myChar = line[2];

    Debug.Assert(opponentChar >= 'A' && opponentChar <= 'C');
    Debug.Assert(myChar >= 'X' && myChar <= 'Z');

    // normalize to 0 for Rock, 1 for Paper, 2 for Scissors
    int opponentMove = opponentChar - 'A';
    int myMove = myChar - 'X';

    partOneScore += CalculateScore(opponentMove, myMove);

    switch (myChar)
    {
        case 'X':
            // must loose
            if (opponentMove == 0)
                myMove = 2;
            else
                myMove = opponentMove - 1;
            break;
        case 'Y':
            // must draw
            myMove = opponentMove;
            break;
        case 'Z':
            // must win
            if (opponentMove == 2)
                myMove = 0;
            else
                myMove = opponentMove + 1;
            break;
        default:
            Debug.Fail(null);
            break;
    }

    partTwoScore += CalculateScore(opponentMove, myMove);
}

Debug.Assert(partOneScore == 17189);

Console.WriteLine($"Day 2 part 1 answer: {partOneScore}");
Console.WriteLine($"Day 2 part 2 answer: {partTwoScore}");

static int CalculateScore(int opponentMove, int myMove)
{
    int roundScore;
    int moveDiff = myMove - opponentMove;

    Debug.Assert(moveDiff >= -2 && moveDiff <= 2);

    // normalize the distance
    if (moveDiff < 0)
        moveDiff += 3;

    if (moveDiff == 0)
    {
        // draw
        roundScore = 3;
    }
    else if (moveDiff == 1)
    {
        // win
        roundScore = 6;
    }
    else
    {
        Debug.Assert(moveDiff == 2);
        // loose
        roundScore = 0;
    }

    roundScore += (myMove + 1);
    return roundScore;
}