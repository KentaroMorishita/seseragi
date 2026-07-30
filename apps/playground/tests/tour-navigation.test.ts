import { describe, expect, test } from "bun:test"
import {
  tourCategories,
  tourChapters,
  tourLessons,
} from "../src/tour/curriculum"
import {
  buildTourNavigationModel,
  tourLessonNeighbors,
} from "../src/tour/navigation"

describe("Tour navigation model", () => {
  test("derives the canonical category, chapter and lesson hierarchy", () => {
    const model = buildTourNavigationModel(
      tourCategories,
      tourChapters,
      tourLessons,
      tourLessons[0]!.id,
      [tourLessons[0]!.id]
    )

    expect(model.categories.map(({ id }) => id)).toEqual(
      tourCategories.map(({ id }) => id)
    )
    expect(model.categories.flatMap(({ chapters }) => chapters)).toHaveLength(
      tourChapters.length
    )
    expect(
      model.categories.flatMap(({ chapters }) =>
        chapters.flatMap(({ lessons }) => lessons)
      )
    ).toHaveLength(tourLessons.length)
    expect(model.progress).toEqual({ completed: 1, total: tourLessons.length })
    expect(model.categories[0]).toMatchObject({
      progress: { completed: 1, total: 1 },
      resumeLessonId: tourLessons[0]!.id,
    })
    expect(model.categories[0]!.chapters[0]!.lessons[0]).toMatchObject({
      id: tourLessons[0]!.id,
      state: "current",
    })
  })

  test("handles one hundred lessons without navigation-specific edits", () => {
    const fixture = largeCurriculumFixture()
    const completedLessonIds = fixture.lessons.slice(0, 57).map(({ id }) => id)
    const currentLesson = fixture.lessons[57]!
    const model = buildTourNavigationModel(
      fixture.categories,
      fixture.chapters,
      fixture.lessons,
      currentLesson.id,
      completedLessonIds
    )

    expect(model.progress).toEqual({ completed: 57, total: 100 })
    expect(model.categories).toHaveLength(10)
    expect(model.categories[5]).toMatchObject({
      progress: { completed: 7, total: 10 },
      resumeLessonId: currentLesson.id,
    })
    expect(model.categories[5]!.chapters[3]).toMatchObject({
      progress: { completed: 1, total: 2 },
    })
    expect(model.categories[5]!.chapters[3]!.lessons[1]!.state).toBe("current")
    expect(model.categories[9]!.chapters[4]!.lessons[1]!.state).toBe(
      "unstarted"
    )
  })

  test("labels previous and next category boundaries", () => {
    const fixture = largeCurriculumFixture()
    const lastLessonInCategory = fixture.lessons[59]!
    const neighbors = tourLessonNeighbors(
      fixture.categories,
      fixture.lessons,
      lastLessonInCategory.id
    )

    expect(neighbors.previous).toMatchObject({
      id: fixture.lessons[58]!.id,
      crossesCategory: false,
    })
    expect(neighbors.next).toMatchObject({
      id: fixture.lessons[60]!.id,
      categoryTitle: "Category 7",
      crossesCategory: true,
    })
  })
})

function largeCurriculumFixture() {
  const categories = Array.from({ length: 10 }, (_, categoryIndex) => ({
    id: `category-${categoryIndex + 1}`,
    title: `Category ${categoryIndex + 1}`,
    summary: `Category ${categoryIndex + 1} overview`,
  }))
  const chapters = categories.flatMap((category) =>
    Array.from({ length: 5 }, (_, chapterIndex) => ({
      id: `${category.id}-chapter-${chapterIndex + 1}`,
      categoryId: category.id,
      title: `Chapter ${chapterIndex + 1}`,
      summary: `Chapter ${chapterIndex + 1} overview`,
    }))
  )
  const lessons = chapters.flatMap((chapter, chapterIndex) =>
    Array.from({ length: 2 }, (_, lessonIndex) => {
      const position = chapterIndex * 2 + lessonIndex + 1
      return {
        id: `lesson-${position}`,
        categoryId: chapter.categoryId,
        chapterId: chapter.id,
        position,
        title: `複数行になりうるlesson title ${position}`,
        goal: `Lesson ${position} goal`,
      }
    })
  )
  return { categories, chapters, lessons }
}
